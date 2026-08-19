#pragma once

#include "esp_err.h"

typedef struct {
  int trig_gpio;
  int echo_gpio;
} ultrasonic_config_t;

esp_err_t ultrasonic_init(const ultrasonic_config_t* config);

esp_err_t ultrasonic_measure_cm(const ultrasonic_config_t* config, float* distance_cm);
