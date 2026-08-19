#pragma once

#include "parking.h"
#include "ultrasonic.h"
#include <stdint.h>

#define PARKING_SLOT_COUNT 3

static const parking_slot_config_t slot_configs[PARKING_SLOT_COUNT] = {
  {
    .id = 1,
    .sensor =
      {
        .trig_gpio = 5,
        .echo_gpio = 18,
      },
    .occupied_threshold_cm = 30.0f,
  },
  {
    .id = 2,
    .sensor =
      {
        .trig_gpio = 19,
        .echo_gpio = 21,
      },
    .occupied_threshold_cm = 30.0f,
  },
  {
    .id = 3,
    .sensor =
      {
        .trig_gpio = 22,
        .echo_gpio = 23,
      },
    .occupied_threshold_cm = 30.0f,
  },
};
