#include <stdio.h>

#include "esp_err.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "esp_log.h"

#include "parking.h"
#include "parking_config.h"
#include "ultrasonic.h"

static const char* TAG = "parking";

static parking_slot_t slots[PARKING_SLOT_COUNT];

static void initialize_slots(void) {
  for (int i = 0; i < PARKING_SLOT_COUNT; i++) {
    ESP_ERROR_CHECK(parking_slot_init(&slots[i], &slot_configs[i]));
  }
}

static void update_slots(void) {
  for (int i = 0; i < PARKING_SLOT_COUNT; i++) {
    float distance_cm;

    esp_err_t result = ultrasonic_measure_cm(&slots[i].sensor, &distance_cm);

    if (result != ESP_OK) {
      ESP_LOGE(TAG, "Slot %d | Sensor error: %s", slots[i].config.id, esp_err_to_name(result));

      slots[i].state = PARKING_ERROR;
      continue;
    }

    parking_event_t event = parking_slot_update(&slots[i], distance_cm);

    switch (event) {
      case PARKING_EVENT_SLOT_OCCUPIED:
        ESP_LOGI(TAG, "Slot %d became OCCUPIED", slots[i].config.id);
        break;
      case PARKING_EVENT_SLOT_FREED:
        ESP_LOGI(TAG, "Slot %d became FREE", slots[i].config.id);
        break;
      case PARKING_EVENT_NONE:
        break;
    }
    ESP_LOGD(TAG,
      "Slot %d | Distance: %.2f cm | State: %s",
      slots[i].config.id,
      slots[i].distance_cm,
      parking_state_to_string(slots[i].state));
  }
}

void app_main(void) {
  printf("\n");
  printf("====================================\n");
  printf("      Smart Parking System\n");
  printf("====================================\n");

  initialize_slots();

  ESP_LOGI(TAG, "Initialized %d parking slots", PARKING_SLOT_COUNT);

  while (1) {
    update_slots();

    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}
