#include <inttypes.h>
#include <stdio.h>

#include "esp_chip_info.h"
#include "esp_flash.h"
#include "esp_system.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "sdkconfig.h"

void app_main(void) {
  printf("\n");
  printf("====================================\n");
  printf("      Smart Parking System\n");
  printf("====================================\n");

  esp_chip_info_t chip_info;
  uint32_t flash_size;

  esp_chip_info(&chip_info);

  printf("Target: %s\n", CONFIG_IDF_TARGET);
  printf("CPU cores: %d\n", chip_info.cores);

  printf("Features: ");

  if (chip_info.features & CHIP_FEATURE_WIFI_BGN) {
    printf("WiFi ");
  }

  if (chip_info.features & CHIP_FEATURE_BT) {
    printf("Bluetooth ");
  }

  if (chip_info.features & CHIP_FEATURE_BLE) {
    printf("BLE ");
  }

  printf("\n");

  unsigned major_rev = chip_info.revision / 100;
  unsigned minor_rev = chip_info.revision % 100;

  printf("Silicon revision: v%d.%d\n", major_rev, minor_rev);

  if (esp_flash_get_size(NULL, &flash_size) == ESP_OK) {
    printf("Flash: %" PRIu32 " MB\n", flash_size / (uint32_t)(1024 * 1024));
  } else {
    printf("Failed to get flash size\n");
  }

  printf("Minimum free heap: %" PRIu32 " bytes\n", esp_get_minimum_free_heap_size());

  printf("System initialized successfully.\n");

  while (1) {
    printf("ESP32 is running...\n");
    vTaskDelay(pdMS_TO_TICKS(1000));
  }
}
