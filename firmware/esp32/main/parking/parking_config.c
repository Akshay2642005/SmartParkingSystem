#include "parking_config.h"

/**
 * Per-slot hardware configuration (GPIO + occupancy thresholds).
 *
 * GPIO mapping matches the Wokwi diagram (diagram.json) and
 * docs/specs/architecture/hardware-architecture.md. Thresholds per
 * docs/specs/decisions/ADR-0004-sensor-selection.md (occupied = 30 cm,
 * free = 35 cm; hysteresis gap prevents state flapping).
 */
const parking_slot_config_t slot_configs[PARKING_SLOT_COUNT] = {
  {
    .id = 1,
    .sensor = {.trig_gpio = 5, .echo_gpio = 18},
    .occupied_threshold_cm = 30.0f,
    .free_threshold_cm = 35.0f,
  },
  {
    .id = 2,
    .sensor = {.trig_gpio = 19, .echo_gpio = 21},
    .occupied_threshold_cm = 30.0f,
    .free_threshold_cm = 35.0f,
  },
  {
    .id = 3,
    .sensor = {.trig_gpio = 22, .echo_gpio = 23},
    .occupied_threshold_cm = 30.0f,
    .free_threshold_cm = 35.0f,
  },
};