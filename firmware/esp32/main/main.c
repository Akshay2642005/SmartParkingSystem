#include <stdio.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "esp_log.h"

#include "parking.h"
#include "ultrasonic.h"

#define TRIG_GPIO 5
#define ECHO_GPIO 18

static const char* TAG = "parking";

void app_main(void) {
  printf("\n[Smart Parking System]\n");

  ultrasonic_config_t ultrasonic_config = {.trigger_gpio = TRIG_GPIO, .echo_gpio = ECHO_GPIO};

  ESP_ERROR_CHECK(ultrasonic_init(&ultrasonic_config));

  ESP_LOGI(TAG, "Parking sensor initialized");

  while (1) {
    float distance_cm;
    esp_err_t result = ultrasonic_measure_distance(&distance_cm);

    if (result == ESP_OK) {
      parking_state_t state = parking_state_from_distance(distance_cm);
      ESP_LOGI(TAG, "Distance: %.2f cm | Slot: %s", distance_cm, parking_state_to_string(state));
    } else {
      ESP_LOGE(TAG, "Failed to read ultrasonic sensor: %s", esp_err_to_name(result));
    }
    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}
