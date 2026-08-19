#pragma once

#include "driver/gpio.h"
#include "esp_err.h"

typedef struct {
  gpio_num_t trig_gpio;
  gpio_num_t echo_gpio;
} ultrasonic_config_t;

typedef struct {
  ultrasonic_config_t config;
} ultrasonic_sensor_t;

esp_err_t ultrasonic_init(ultrasonic_sensor_t* sensor, const ultrasonic_config_t* config);

esp_err_t ultrasonic_measure_cm(const ultrasonic_sensor_t* sensor, float* distance_cm);
