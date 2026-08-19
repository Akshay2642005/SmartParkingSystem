#include "ultrasonic.h"

#include "esp_log.h"
#include "esp_timer.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

static const char* TAG = "ultrasonic";

#define ULTRASONIC_TIMEOUT_US 30000
#define SOUND_SPEED_CM_PER_US 0.0343f

esp_err_t ultrasonic_init(ultrasonic_sensor_t* sensor, const ultrasonic_config_t* config) {
  if (sensor == NULL || config == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  sensor->config = *config;

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

  ESP_ERROR_CHECK(gpio_set_level(config->trig_gpio, 0));

  ESP_LOGI(TAG, "Initialized TRIG=%d ECHO=%d", config->trig_gpio, config->echo_gpio);

  return ESP_OK;
}

esp_err_t ultrasonic_measure_cm(const ultrasonic_sensor_t* sensor, float* distance_cm) {
  if (sensor == NULL || distance_cm == NULL) {
    return ESP_ERR_INVALID_ARG;
  }

  const gpio_num_t trig = sensor->config.trig_gpio;
  const gpio_num_t echo = sensor->config.echo_gpio;

  gpio_set_level(trig, 0);
  esp_rom_delay_us(2);

  gpio_set_level(trig, 1);
  esp_rom_delay_us(10);
  gpio_set_level(trig, 0);

  int64_t timeout_start = esp_timer_get_time();

  while (gpio_get_level(echo) == 0) {
    if (esp_timer_get_time() - timeout_start >= ULTRASONIC_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  int64_t pulse_start = esp_timer_get_time();

  while (gpio_get_level(echo) == 1) {
    if (esp_timer_get_time() - pulse_start >= ULTRASONIC_TIMEOUT_US) {
      return ESP_ERR_TIMEOUT;
    }
  }

  int64_t pulse_end = esp_timer_get_time();

  int64_t pulse_duration_us = pulse_end - pulse_start;

  *distance_cm = (pulse_duration_us * SOUND_SPEED_CM_PER_US) / 2.0f;

  return ESP_OK;
}
