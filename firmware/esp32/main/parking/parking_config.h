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

/** Slot table defined in parking_config.c. */
extern const parking_slot_config_t slot_configs[PARKING_SLOT_COUNT];