#include "ultrasonic.h"

#include "driver/gpio.h"
#include "esp_rom_sys.h"
#include "esp_timer.h"

#define ECHO_TIMEOUT_US 30000

esp_err_t ultrasonic_init(const ultrasonic_config_t* config) {
  if (config == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  gpio_config_t trig_config = {
      .pin_bit_mask = 1ULL << config->trig_gpio,
      .mode = GPIO_MODE_OUTPUT,
      .pull_up_en = GPIO_PULLUP_DISABLE,
      .pull_down_en = GPIO_PULLDOWN_DISABLE,
      .intr_type = GPIO_INTR_DISABLE,
  };

  ESP_ERROR_CHECK(gpio_config(&trig_config));

  gpio_config_t echo_config = {
      .pin_bit_mask = 1ULL << config->echo_gpio,
      .mode = GPIO_MODE_INPUT,
      .pull_up_en = GPIO_PULLUP_DISABLE,
      .pull_down_en = GPIO_PULLDOWN_DISABLE,
      .intr_type = GPIO_INTR_DISABLE,
  };

  ESP_ERROR_CHECK(gpio_config(&echo_config));

  gpio_set_level(config->trig_gpio, 0);

  return ESP_OK;
}

esp_err_t ultrasonic_measure_cm(const ultrasonic_config_t* config, float* distance_cm) {
  if (config == NULL || distance_cm == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  gpio_set_level(config->trig_gpio, 0);
  esp_rom_delay_us(2);

  gpio_set_level(config->trig_gpio, 1);
  esp_rom_delay_us(10);
  gpio_set_level(config->trig_gpio, 0);

  int64_t start = esp_timer_get_time();

  while (gpio_get_level(config->echo_gpio) == 0) {
    if (esp_timer_get_time() - start > ECHO_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  int64_t echo_start = esp_timer_get_time();

  while (gpio_get_level(config->echo_gpio) == 1) {
    if (esp_timer_get_time() - echo_start > ECHO_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  int64_t echo_end = esp_timer_get_time();

  int64_t duration_us = echo_end - echo_start;

  *distance_cm = duration_us / 58.0f;

  return ESP_OK;
}
