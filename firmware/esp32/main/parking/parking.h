#pragma once

#include <stddef.h>
#include <stdint.h>

#include "esp_err.h"
#include "ultrasonic.h"

/**
 * Parking domain: slots, the occupancy state machine, and lot statistics.
 *
 * Spec: docs/specs/decisions/ADR-0007-parking-state-model.md,
 *       docs/specs/product/REQUIREMENTS.md (FR-001..FR-003, FR-010..FR-012),
 *       docs/specs/architecture/firmware-architecture.md.
 */

/** Occupancy state of a single parking slot. */
typedef enum {
    PARKING_FREE,     /**< Slot is available. */
    PARKING_OCCUPIED, /**< Slot is in use. */
    PARKING_ERROR,    /**< Sensor measurement failed; state is unknown. */
} parking_state_t;

/** Events emitted when a slot changes occupancy state. */
typedef enum {
    PARKING_EVENT_NONE,          /**< No state change this update. */
    PARKING_EVENT_SLOT_OCCUPIED, /**< Slot transitioned FREE -> OCCUPIED. */
    PARKING_EVENT_SLOT_FREED,    /**< Slot transitioned OCCUPIED -> FREE. */
} parking_event_type_t;

/**
 * Occupancy event with full context.
 * Produced by parking_slot_update(); consumed by the caller.
 * Spec: docs/specs/product/GLOSSARY.yaml (parking_event),
 *       docs/specs/decisions/ADR-0007-parking-state-model.md (Events).
 */
typedef struct {
    parking_event_type_t type; /**< Type of event. */
    uint8_t slot_id;           /**< Slot identifier (copied from config). */
    float distance_cm;         /**< Distance that triggered the event. */
} parking_event_t;

/** Per-slot configuration (see parking_config.c). */
typedef struct {
    uint8_t id;                  /**< Stable slot identifier. */
    ultrasonic_config_t sensor;  /**< GPIO wiring for this slot's sensor. */
    float occupied_threshold_cm; /**< Distance <= this => OCCUPIED. */
    float free_threshold_cm;     /**< Distance >= this => FREE. */
} parking_slot_config_t;

/** Runtime state of a single parking slot. */
typedef struct {
    parking_slot_config_t config;       /**< Slot configuration (copied at init). */
    ultrasonic_sensor_t sensor;         /**< Sensor instance, initialized at slot init. */
    parking_state_t state;              /**< Current occupancy state. */
    parking_state_t state_before_error; /**< Stable state to restore after ERROR recovery. */
    float distance_cm;                  /**< Latest measured distance. */
} parking_slot_t;

/** Aggregate view over all slots in the lot. */
typedef struct {
    parking_slot_t* slots;  /**< Pointer to the statically allocated slot array. */
    size_t slot_count;      /**< Number of slots in the array. */
    size_t occupied_count;  /**< Slots currently OCCUPIED. */
    size_t available_count; /**< Slots currently FREE. */
    size_t error_count;     /**< Slots currently in PARKING_ERROR. */
} parking_lot_t;

/**
 * Initialize a slot: copy its config, initialize its sensor, and set state to
 * PARKING_FREE. FR-001 (slot identification).
 *
 * @param slot   Slot to initialize.
 * @param config Configuration to copy; must not be NULL.
 * @return ESP_OK on success, ESP_ERR_INVALID_ARG on bad arguments, or an error
 *         from ultrasonic_init().
 */
esp_err_t parking_slot_init(parking_slot_t* slot, const parking_slot_config_t* config);

/**
 * Feed one measured distance into the slot state machine (FR-002, FR-011).
 *
 * Transition rules (ADR-0007):
 *   FREE:      distance <= occupied_threshold_cm -> OCCUPIED (SLOT_OCCUPIED)
 *   OCCUPIED:  distance >= free_threshold_cm     -> FREE     (SLOT_FREED)
 *   ERROR:     first valid measurement decides:
 *                <= occupied_threshold_cm        -> OCCUPIED
 *                >= free_threshold_cm            -> FREE
 *                hysteresis band                 -> restore state_before_error
 *              An event is emitted only when the recovered state differs
 *              from state_before_error (a real occupancy change).
 * Hysteresis between the two thresholds prevents flapping on noise.
 *
 * @param slot        Slot to update.
 * @param distance_cm Validated distance in centimeters (see
 *                    parking_measurement_is_valid); callers must not pass
 *                    unvalidated sensor output.
 *
 * @return Event describing the state change. type == PARKING_EVENT_NONE means
 *         no transition occurred; ERROR transitions never produce occupancy
 *         events, and recovery emits an event only for a real occupancy change
 *         (ADR-0007).
 */
parking_event_t parking_slot_update(parking_slot_t* slot, float distance_cm);

/**
 * Mark a slot as PARKING_ERROR (e.g. sensor measurement failure). FR-012.
 *
 * @param slot Slot to mark.
 */
void parking_slot_mark_error(parking_slot_t* slot);

/**
 * Initialize a parking lot over a statically allocated slot array.
 *
 * @param lot        Lot to initialize.
 * @param slots      Slot array (must remain valid for the lot's lifetime).
 * @param slot_count Number of slots in @p slots.
 */
void parking_lot_init(parking_lot_t* lot, parking_slot_t* slots, size_t slot_count);

/**
 * Recompute lot statistics from slot states.
 *
 * Maintains the invariant total = occupied + available + error; ERROR slots
 * are excluded from available_count because they are not bookable.
 *
 * @param lot Lot to update.
 */
void parking_lot_update_counts(parking_lot_t* lot);

/**
 * Scan all slots: measure each sensor, update state, handle failures, and
 * refresh lot statistics. FR-010 (periodic sampling).
 *
 * One failing sensor does not abort the scan; the failing slot is marked
 * PARKING_ERROR and remaining slots are still processed.
 *
 * @param lot Lot to scan.
 * @return ESP_OK after a full scan, ESP_ERR_INVALID_ARG if @p lot is invalid.
 */
esp_err_t parking_lot_scan(parking_lot_t* lot);

/*
 * Lot query API: const-correct read views over the counters recomputed by
 * parking_lot_update_counts() after each scan. Counts are size_t to match
 * slot_count and array indexing; fixed-width mapping belongs to the telemetry
 * encoder once networking lands.
 */

/** @return Total number of slots in the lot. */
size_t parking_lot_get_total(const parking_lot_t* lot);

/** @return Number of OCCUPIED slots. */
size_t parking_lot_get_occupied(const parking_lot_t* lot);

/** @return Number of FREE (bookable) slots; ERROR slots are excluded. */
size_t parking_lot_get_available(const parking_lot_t* lot);

/** @return Number of slots currently in ERROR state. */
size_t parking_lot_get_error(const parking_lot_t* lot);

/** @return Human-readable name for a parking state. */
const char* parking_state_to_string(parking_state_t state);
