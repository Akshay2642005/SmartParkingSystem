#include "ultrasonic.h"
#include "driver/gpio.h"
#include "esp_err.h"
#include "esp_rom_sys.h"
#include "esp_timer.h"
#include "hal/gpio_types.h"
#include <stdlib.h>

#define ECHO_TIMEOUT_US 30000

static ultrasonic_config_t sensor;

esp_err_t ultrasonic_init(const ultrasonic_config_t *config) {
  if (config == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  sensor = *config;

  gpio_config_t trig_config = {
    .pin_bit_mask = 1ULL << sensor.trigger_gpio,
    .mode = GPIO_MODE_OUTPUT,
    .pull_up_en = GPIO_PULLUP_DISABLE,
    .pull_down_en = GPIO_PULLDOWN_DISABLE,
    .intr_type = GPIO_INTR_DISABLE,
  };

  ESP_ERROR_CHECK(gpio_config(&trig_config));

  gpio_config_t echo_config = {
    .pin_bit_mask = 1ULL << sensor.echo_gpio,
    .mode = GPIO_MODE_INPUT,
    .pull_up_en = GPIO_PULLUP_DISABLE,
    .pull_down_en = GPIO_PULLDOWN_DISABLE,
    .intr_type = GPIO_INTR_DISABLE,
  };

  ESP_ERROR_CHECK(gpio_config(&echo_config));

  gpio_set_level(sensor.trigger_gpio, 0);

  return ESP_OK;
}

esp_err_t ultrasonic_measure_distance(float *distance_cm) {
  if (distance_cm == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  gpio_set_level(sensor.trigger_gpio, 0);
  esp_rom_delay_us(2);

  gpio_set_level(sensor.trigger_gpio,1);
  esp_rom_delay_us(10);

  gpio_set_level(sensor.trigger_gpio, 0);

  uint32_t start = esp_timer_get_time();

  while (gpio_get_level(sensor.echo_gpio) == 0) {
    if ((esp_timer_get_time() - start) > ECHO_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  uint32_t echo_start = esp_timer_get_time();

  while (gpio_get_level(sensor.echo_gpio) == 1) {
    if ((esp_timer_get_time() - echo_start) > ECHO_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  uint32_t echo_end = esp_timer_get_time();

  // HC-SR04: distance (cm) ~= duration / 58.0f
  *distance_cm = (echo_end - echo_start) / 58.0f;

  return ESP_OK;
}
