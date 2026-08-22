#include "parking.h"

#include <math.h>
#include <stdbool.h>

#include "esp_log.h"

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
    if (!isfinite(distance_cm)) {
        return false;
    }

    if (distance_cm <= 0.0f) {
        return false;
    }

    return distance_cm >= ULTRASONIC_MIN_DISTANCE_CM && distance_cm <= ULTRASONIC_MAX_DISTANCE_CM;
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

esp_err_t parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config) {
    if (slot == NULL || config == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    slot->config = *config;
    slot->state = PARKING_FREE;
    slot->state_before_error = PARKING_FREE;
    slot->distance_cm = 0.0f;

    // Initialize the slot's sensor; failure here is a fatal configuration error.
    return ultrasonic_init(&slot->sensor, &slot->config.sensor);
}

parking_event_t parking_slot_update(parking_slot_t* slot, float distance_cm) {
    if (slot == NULL) {
        return (parking_event_t){PARKING_EVENT_NONE, 0, 0.0f};
    }

    slot->distance_cm = distance_cm;

    // ADR-0007 transition rules with hysteresis:
    //   FREE:     distance <= occupied_threshold_cm -> OCCUPIED
    //   OCCUPIED: distance >= free_threshold_cm     -> FREE
    //   ERROR:    first valid measurement recovers (see case below)
    switch (slot->state) {
        case PARKING_FREE:
            if (distance_cm <= slot->config.occupied_threshold_cm) {
                slot->state = PARKING_OCCUPIED;
                return make_parking_event(PARKING_EVENT_SLOT_OCCUPIED, slot);
            }
            break;

        case PARKING_OCCUPIED:
            if (distance_cm >= slot->config.free_threshold_cm) {
                slot->state = PARKING_FREE;
                return make_parking_event(PARKING_EVENT_SLOT_FREED, slot);
            }
            break;

        case PARKING_ERROR: {
            // Deterministic recovery from the first valid measurement.
            parking_state_t recovered;

            if (distance_cm <= slot->config.occupied_threshold_cm) {
                recovered = PARKING_OCCUPIED;
            } else if (distance_cm >= slot->config.free_threshold_cm) {
                recovered = PARKING_FREE;
            } else {
                // Ambiguous hysteresis band: restore the pre-error state, no guess.
                recovered = slot->state_before_error;
            }

            slot->state = recovered;

            // Only a genuine occupancy change produces an event; recovering to the
            // state the slot already had is silent.
            if (recovered != slot->state_before_error) {
                return make_parking_event(recovered == PARKING_OCCUPIED ? PARKING_EVENT_SLOT_OCCUPIED : PARKING_EVENT_SLOT_FREED, slot);
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

        parking_event_t event = parking_slot_update(slot, distance_cm);
        handle_parking_event(&event);

        // Event-based INFO logging; detailed readings stay at DEBUG level.
        ESP_LOGD(TAG,
            "Slot %d | Distance: %.2f cm | State: %s",
            slot->config.id,
            slot->distance_cm,
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
