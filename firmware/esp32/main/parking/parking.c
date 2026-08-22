#include "parking.h"

#include <stdbool.h>

#include "esp_log.h"
#include "parking_config.h"

/**
 * Parking domain implementation.
 *
 * Spec: docs/specs/decisions/ADR-0007-parking-state-model.md,
 *       docs/specs/product/REQUIREMENTS.md (FR-001..FR-003, FR-010..FR-012).
 */

static const char* TAG = "parking";

/**
 * Validate a raw measurement before it may influence parking state.
 *
 * Rejects non-numeric values, zero/negative distances, and readings outside
 * the HC-SR04 supported range. Invalid measurements must never be converted
 * into FREE or OCCUPIED.
 *
 * Spec: docs/specs/product/REQUIREMENTS.md (FR-012),
 *       docs/specs/decisions/ADR-0004-sensor-selection.md (driver limits).
 */
static bool parking_measurement_is_valid(float distance_cm) {
    // Plausibility policy (finite, positive, within sensor range incl. the
    // jitter tolerance below the datasheet floor) lives in the sensor layer.
    return ultrasonic_distance_is_plausible(distance_cm);
}

/**
 * Feed a validated raw distance through the per-slot EMA filter.
 *
 * The first valid reading seeds the filter directly (no ramp-up from zero).
 * Invalid readings never reach this function, so filter state cannot be
 * corrupted by sensor failures; ERROR periods deliberately do not reset the
 * filter — the last smoothed value is the best available prior on recovery.
 *
 * Spec: docs/specs/architecture/firmware-architecture.md (measurement
 * pipeline), docs/specs/decisions/ADR-0004-sensor-selection.md.
 */
static float parking_filter_distance(parking_slot_t* slot, float raw_cm) {
    if (!slot->filter_seeded) {
        slot->filter_seeded = true;
        slot->filtered_distance_cm = raw_cm;
        return raw_cm;
    }

    slot->filtered_distance_cm =
        PARKING_FILTER_ALPHA * raw_cm + (1.0f - PARKING_FILTER_ALPHA) * slot->filtered_distance_cm;

    return slot->filtered_distance_cm;
}

/**
 * Centralizes event construction so every transition site carries the same
 * context (slot id + measurement that caused the change).
 */
static parking_event_t make_parking_event(parking_event_type_t type, const parking_slot_t* slot) {
    parking_event_t event = {
        .type = type,
        .slot_id = slot->config.id,
        .distance_cm = slot->distance_cm,
    };
    return event;
}

/**
 * Handle one parking event.
 *
 * Single seam between the state machine and its consumers: logging today;
 * later consumers (event queue, networking) attach here, never inside
 * parking_slot_update().
 *
 * Spec: docs/specs/architecture/firmware-architecture.md (Events).
 */
static void handle_parking_event(const parking_event_t* event) {
    switch (event->type) {
        case PARKING_EVENT_SLOT_OCCUPIED:
            ESP_LOGI(TAG, "Slot %d became OCCUPIED", event->slot_id);
            break;

        case PARKING_EVENT_SLOT_FREED:
            ESP_LOGI(TAG, "Slot %d became FREE", event->slot_id);
            break;

        case PARKING_EVENT_NONE:
            break;
    }
}

/**
 * Classify one filtered reading against slot thresholds (hysteresis).
 *
 * Returns PARKING_ERROR as the "ambiguous hysteresis band" marker for the
 * debouncer — not as an error condition.
 *
 * Spec: docs/specs/decisions/ADR-0007-parking-state-model.md.
 */
static parking_state_t parking_classify(const parking_slot_t* slot, float distance_cm) {
    if (distance_cm <= slot->config.occupied_threshold_cm) {
        return PARKING_OCCUPIED;
    }

    if (distance_cm >= slot->config.free_threshold_cm) {
        return PARKING_FREE;
    }

    return PARKING_ERROR;
}

esp_err_t parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config) {
    if (slot == NULL || config == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    slot->config = *config;
    slot->state = PARKING_FREE;
    slot->state_before_error = PARKING_FREE;
    slot->distance_cm = 0.0f;
    slot->filtered_distance_cm = 0.0f;
    slot->filter_seeded = false;
    slot->pending_state = PARKING_FREE;
    slot->confirmation_count = 0;

    // Initialize the slot's sensor; failure here is a fatal configuration error.
    return ultrasonic_init(&slot->sensor, &slot->config.sensor);
}

parking_event_t parking_slot_update(parking_slot_t* slot, float distance_cm) {
    if (slot == NULL) {
        return (parking_event_t){PARKING_EVENT_NONE, 0, 0.0f};
    }

    slot->distance_cm = distance_cm;

    const parking_state_t candidate = parking_classify(slot, distance_cm);

    // ADR-0007 transition rules:
    //   FREE/OCCUPIED: a transition requires PARKING_*_CONFIRMATION_COUNT
    //     consecutive readings of the same candidate state; an ambiguous
    //     hysteresis-band reading resets the count (no evidence either way).
    //   ERROR: recovery is deliberately not debounced — the slot was blind
    //     during the failure, so the first valid measurement decides.
    switch (slot->state) {
        case PARKING_FREE:
        case PARKING_OCCUPIED: {
            if (candidate == PARKING_ERROR || candidate == slot->state) {
                // Ambiguous or agrees with current state: nothing to confirm.
                slot->pending_state = PARKING_FREE;
                slot->confirmation_count = 0;
                break;
            }

            if (candidate == slot->pending_state) {
                slot->confirmation_count++;
            } else {
                slot->pending_state = candidate;
                slot->confirmation_count = 1;
            }

            const uint8_t required = (candidate == PARKING_OCCUPIED)
                                         ? PARKING_OCCUPIED_CONFIRMATION_COUNT
                                         : PARKING_FREE_CONFIRMATION_COUNT;

            if (slot->confirmation_count < required) {
                break;
            }

            // Confirmed: fire the transition.
            slot->state = candidate;
            slot->pending_state = PARKING_FREE;
            slot->confirmation_count = 0;

            return make_parking_event(
                candidate == PARKING_OCCUPIED ? PARKING_EVENT_SLOT_OCCUPIED : PARKING_EVENT_SLOT_FREED,
                slot);
        }

        case PARKING_ERROR: {
            // Deterministic recovery from the first valid measurement. An
            // ambiguous hysteresis band restores the pre-error state, no guess.
            parking_state_t recovered =
                (candidate == PARKING_ERROR) ? slot->state_before_error : candidate;

            slot->state = recovered;
            slot->pending_state = PARKING_FREE;
            slot->confirmation_count = 0;

            // Only a genuine occupancy change produces an event; recovering to
            // the state the slot already had is silent.
            if (recovered != slot->state_before_error) {
                return make_parking_event(
                    recovered == PARKING_OCCUPIED ? PARKING_EVENT_SLOT_OCCUPIED : PARKING_EVENT_SLOT_FREED,
                    slot);
            }

            break;
        }
    }

    return (parking_event_t){PARKING_EVENT_NONE, 0, 0.0f};
}

void parking_slot_mark_error(parking_slot_t* slot) {
    if (slot == NULL) {
        return;
    }

    // Idempotent: remember the last stable state only on the first failure,
    // so repeated scan failures cannot overwrite the recovery target.
    if (slot->state != PARKING_ERROR) {
        slot->state_before_error = slot->state;
        slot->state = PARKING_ERROR;
    }
}

const char* parking_state_to_string(parking_state_t state) {
    switch (state) {
        case PARKING_FREE:
            return "FREE";

        case PARKING_OCCUPIED:
            return "OCCUPIED";

        case PARKING_ERROR:
            return "ERROR";

        default:
            return "UNKNOWN";
    }
}

void parking_lot_init(parking_lot_t* lot, parking_slot_t* slots, size_t slot_count) {
    if (lot == NULL) {
        return;
    }

    lot->slots = slots;
    lot->slot_count = slot_count;
    lot->occupied_count = 0;
    lot->error_count = 0;
    lot->available_count = slot_count;

    // Counts are recomputed from the (already initialized) slot states.
    parking_lot_update_counts(lot);
}

void parking_lot_update_counts(parking_lot_t* lot) {
    if (lot == NULL || lot->slots == NULL) {
        return;
    }

    lot->occupied_count = 0;
    lot->error_count = 0;
    for (size_t i = 0; i < lot->slot_count; i++) {
        switch (lot->slots[i].state) {
            case PARKING_OCCUPIED:
                lot->occupied_count++;
                break;

            case PARKING_ERROR:
                lot->error_count++;
                break;

            case PARKING_FREE:
                break;
        }
    }

    // Invariant: total = occupied + available + error. ERROR slots are not
    // bookable, so they must never appear as available.
    lot->available_count = lot->slot_count - lot->occupied_count - lot->error_count;
}

esp_err_t parking_lot_scan(parking_lot_t* lot) {
    if (lot == NULL || lot->slots == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    for (size_t i = 0; i < lot->slot_count; i++) {
        parking_slot_t* slot = &lot->slots[i];
        float distance_cm;

        esp_err_t result = ultrasonic_measure_cm(&slot->sensor, &distance_cm);

        if (result != ESP_OK) {
            // Recoverable runtime error: mark the slot and keep scanning the rest.
            parking_slot_mark_error(slot);
            ESP_LOGE(TAG, "Slot %d | Sensor error: %s", slot->config.id, esp_err_to_name(result));
            continue;
        }

        if (!parking_measurement_is_valid(distance_cm)) {
            // Defense-in-depth: never feed invalid data into the state machine.
            parking_slot_mark_error(slot);
            ESP_LOGE(TAG, "Slot %d | Invalid measurement: %.2f cm", slot->config.id, distance_cm);
            continue;
        }

        const float stable_cm = parking_filter_distance(slot, distance_cm);

        parking_event_t event = parking_slot_update(slot, stable_cm);
        handle_parking_event(&event);

        // Event-based INFO logging; detailed readings stay at DEBUG level.
        ESP_LOGD(TAG,
            "Slot %d | Raw: %.2f cm | Stable: %.2f cm | State: %s",
            slot->config.id,
            distance_cm,
            slot->filtered_distance_cm,
            parking_state_to_string(slot->state));
    }

    parking_lot_update_counts(lot);

    ESP_LOGI(TAG,
        "Parking Lot | Total: %zu | Occupied: %zu | Available: %zu | Errors: %zu",
        parking_lot_get_total(lot),
        parking_lot_get_occupied(lot),
        parking_lot_get_available(lot),
        parking_lot_get_error(lot));

    return ESP_OK;
}

size_t parking_lot_get_total(const parking_lot_t* lot) {
    if (lot == NULL) {
        return 0;
    }

    return lot->slot_count;
}

size_t parking_lot_get_occupied(const parking_lot_t* lot) {
    if (lot == NULL) {
        return 0;
    }

    return lot->occupied_count;
}

size_t parking_lot_get_available(const parking_lot_t* lot) {
    if (lot == NULL) {
        return 0;
    }

    return lot->available_count;
}

size_t parking_lot_get_error(const parking_lot_t* lot) {
    if (lot == NULL) {
        return 0;
    }

    return lot->error_count;
}

parking_state_t parking_slot_get_state(const parking_slot_t* slot) {
    if (slot == NULL) {
        return PARKING_ERROR;
    }

    return slot->state;
}

float parking_slot_get_distance_cm(const parking_slot_t* slot) {
    if (slot == NULL) {
        return 0.0f;
    }

    return slot->filtered_distance_cm;
}
