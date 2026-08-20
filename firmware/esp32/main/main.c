#include <stdio.h>

#include "esp_err.h"
#include "esp_log.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "parking.h"
#include "parking_config.h"

/**
 * Application entry point — orchestration only.
 *
 * Spec: docs/specs/architecture/firmware-architecture.md.
 * The parking domain (parking.c) owns scanning, state transitions, and
 * statistics; main.c initializes the hardware and drives the scan loop.
 */

static const char* TAG = "parking";

/** Statically allocated slots and the lot view over them. */
static parking_slot_t slots[PARKING_SLOT_COUNT];
static parking_lot_t parking_lot;

/** Initialize every slot's sensor and register them with the lot. */
static void initialize_slots(void) {
  for (int i = 0; i < PARKING_SLOT_COUNT; i++) {
    // Sensor init failure is a fatal configuration problem -> abort.
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

  // Periodic scan loop. The 1 s interval is scheduled for configuration
  // (Phase 13); polling stays here until the dedicated task lands (Phase 14).
  while (1) {
    ESP_ERROR_CHECK(parking_lot_scan(&parking_lot));

    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}