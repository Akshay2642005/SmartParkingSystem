#include "parking_config.h"

/**
 * Per-slot hardware configuration (GPIO + occupancy thresholds).
 *
 * GPIO mapping matches the Wokwi diagram (diagram.json) and
 * docs/specs/architecture/hardware-architecture.md. Thresholds per
 * docs/specs/decisions/ADR-0004-sensor-selection.md; the hysteresis gap
 * between them prevents state flapping. They depend on sensor mounting
 * height, so they stay with the hardware table rather than parking_settings.h.
 */

/** Distance (cm) at or below which a slot counts as OCCUPIED. */
#define SLOT_OCCUPIED_THRESHOLD_CM 30.0f

/** Distance (cm) at or above which a slot counts as FREE. */
#define SLOT_FREE_THRESHOLD_CM 35.0f

const parking_slot_config_t slot_configs[PARKING_SLOT_COUNT] = {
    {
        .id = 1,
        .sensor = {.trig_gpio = 5, .echo_gpio = 18},
        .occupied_threshold_cm = SLOT_OCCUPIED_THRESHOLD_CM,
        .free_threshold_cm = SLOT_FREE_THRESHOLD_CM,
    },
    {
        .id = 2,
        .sensor = {.trig_gpio = 19, .echo_gpio = 21},
        .occupied_threshold_cm = SLOT_OCCUPIED_THRESHOLD_CM,
        .free_threshold_cm = SLOT_FREE_THRESHOLD_CM,
    },
    {
        .id = 3,
        .sensor = {.trig_gpio = 22, .echo_gpio = 23},
        .occupied_threshold_cm = SLOT_OCCUPIED_THRESHOLD_CM,
        .free_threshold_cm = SLOT_FREE_THRESHOLD_CM,
    },
};
