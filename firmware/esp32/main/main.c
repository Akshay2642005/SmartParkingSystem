#include <stdio.h>

#include "esp_err.h"
#include "esp_log.h"

#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "debug_console.h"
#if CONFIG_PARKING_MQTT_ENABLE
#include "mqtt_publisher.h"
#endif
#include "parking.h"
#include "parking_config.h"
#include "parking_settings.h"

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

/**
 * Periodic scan task — owns the full parking lot cycle.
 *
 * Runs forever at PARKING_SCAN_INTERVAL_MS with absolute scheduling
 * (vTaskDelayUntil), so the scan rate stays deterministic regardless of how
 * long each scan takes. Any sensor failure is non-fatal inside
 * parking_lot_scan(); a hard ESP_ERROR_CHECK here means an unexpected bug.
 *
 * Created by app_main; never returns.
 */
static void parking_task(void* arg) {
    (void)arg;
    TickType_t last_wake_time = xTaskGetTickCount();
    uint32_t scan_count = 0;

    while (1) {
        ESP_ERROR_CHECK(parking_lot_scan(&parking_lot));

        // One-shot stack headroom probe after the first few scans, once the
        // deepest call path (scan + logging) has executed. High water mark is
        // in words (4 B on Xtensa); logged at DEBUG to keep steady-state quiet.
        if (++scan_count == PARKING_STACK_PROBE_AFTER_SCANS) {
            ESP_LOGD(TAG,
                "Parking task stack high water mark: %u words free",
                (unsigned)uxTaskGetStackHighWaterMark(NULL));
        }

        vTaskDelayUntil(&last_wake_time, pdMS_TO_TICKS(PARKING_SCAN_INTERVAL_MS));
    }
}

void app_main(void) {
    printf("\n");
    printf("====================================\n");
    printf("      Smart Parking System\n");
    printf("====================================\n");

    initialize_slots();

    // Telemetry registers its transition observer before the parking task
    // exists, so no committed transition can be missed (communication.md).
#if CONFIG_PARKING_MQTT_ENABLE
    ESP_ERROR_CHECK(parking_mqtt_publisher_start());
#endif

    ESP_LOGI(TAG, "Initialized %d parking slots", PARKING_SLOT_COUNT);

    BaseType_t created = xTaskCreate(parking_task,
        PARKING_TASK_NAME,
        PARKING_TASK_STACK_SIZE,
        NULL,
        PARKING_TASK_PRIORITY,
        NULL);

    ESP_ERROR_CHECK(created == pdPASS ? ESP_OK : ESP_FAIL);

    debug_console_start();
}
