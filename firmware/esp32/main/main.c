#include <stdio.h>

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "esp_log.h"

#include "parking.h"
#include "parking_config.h"
#include "ultrasonic.h"

#define SLOT_COUNT 3
static const char* TAG = "parking";

static parking_slot_t slots[SLOT_COUNT];

#include "parking.h"

static parking_slot_t slots[PARKING_SLOT_COUNT];

static void initialize_slots(void) {
  for (int i = 0; i < PARKING_SLOT_COUNT; i++) {
    parking_slot_init(&slots[i], &slot_configs[i]);

    ESP_ERROR_CHECK(ultrasonic_init(&slots[i].config.sensor));
  }
}

static void update_slots(void) {
  for (int i = 0; i < SLOT_COUNT; i++) {
    float distance_cm;

    esp_err_t result = ultrasonic_measure_cm(&slots[i].config.sensor, &distance_cm);

    if (result == ESP_OK) {
      parking_slot_update(&slots[i], distance_cm);

      ESP_LOGI(TAG,
        "Slot %d | Distance: %.2f cm | State: %s",
        slots[i].config.id,
        slots[i].distance_cm,
        parking_state_to_string(slots[i].state));
    } else {
      slots[i].state = PARKING_ERROR;

      ESP_LOGE(TAG, "Slot %d | Sensor error: %s", slots[i].config.id, esp_err_to_name(result));
    }
  }
}

void app_main(void) {
  printf("\n");
  printf("====================================\n");
  printf("      Smart Parking System\n");
  printf("====================================\n");

  initialize_slots();

  ESP_LOGI(TAG, "Initialized %d parking slots", SLOT_COUNT);

  while (1) {
    update_slots();

    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}
