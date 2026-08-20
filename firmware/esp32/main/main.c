#include <stdio.h>

#include "esp_err.h"
#include "esp_log.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "parking.h"
#include "parking_config.h"

static const char* TAG = "parking";

static parking_slot_t slots[PARKING_SLOT_COUNT];
static parking_lot_t parking_lot;

static void initialize_slots(void) {
  for (int i = 0; i < PARKING_SLOT_COUNT; i++) {
    ESP_ERROR_CHECK(parking_slot_init(&slots[i], &slot_configs[i]));
  }

  parking_lot_init(&parking_lot, slots, PARKING_SLOT_COUNT);
}

void app_main(void) {
  printf("\n");
  printf("====================================\n");
  printf("      Smart Parking System\n");
  printf("====================================\n");

  initialize_slots();

  ESP_LOGI(TAG, "Initialized %d parking slots", PARKING_SLOT_COUNT);

  while (1) {
    ESP_ERROR_CHECK(parking_lot_scan(&parking_lot));

    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}