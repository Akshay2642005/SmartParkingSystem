#pragma once

#include "parking.h"

/**
 * Parking slot configuration.
 *
 * The actual slot table lives in parking_config.c so the header only declares
 * the extern table. Spec: docs/specs/decisions/ADR-0004-sensor-selection.md,
 *       docs/specs/architecture/hardware-architecture.md.
 */

/** Number of parking slots / ultrasonic sensors on this device. */
#define PARKING_SLOT_COUNT 3

/**
 * EMA smoothing factor for distance stability, in the range (0..1].
 *
 * Higher values react faster to real changes; lower values smooth more.
 * With a 1 s scan period, 0.3 settles ~90 % within about six scans — fast
 * enough for a car parking, slow enough to swallow echo jitter.
 */
#define PARKING_FILTER_ALPHA 0.3f

/**
 * Consecutive readings of the same candidate state required before an
 * occupancy transition fires. Confirmation counting is signal-processing
 * policy shared by all identical sensors, hence global rather than per-slot.
 */
#define PARKING_OCCUPIED_CONFIRMATION_COUNT 2
#define PARKING_FREE_CONFIRMATION_COUNT 2

/** Slot table defined in parking_config.c. */
extern const parking_slot_config_t slot_configs[PARKING_SLOT_COUNT];
