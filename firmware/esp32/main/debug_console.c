#include <stdio.h>
#include <string.h>

#include "esp_err.h"
#include "esp_log.h"

#include "driver/uart.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"

#include "debug_console.h"
#include "parking.h"
#include "parking_config.h"

#if CONFIG_PARKING_DEBUG_INJECT

/**
 * Debug console (debug builds only): reads "setdist <slot-id> <cm>" lines
 * from UART0 and queues them as one-shot measurement injections for the next
 * parking_lot_scan() of the given slot. Exists solely so wokwi-cli scenarios
 * can drive occupancy/error paths headlessly via write-serial; production
 * builds compile this file out entirely.
 */

static const char* TAG = "dbg_console";

/** UART the IDF console runs on; matches the Wokwi serial monitor wiring. */
#define DEBUG_CONSOLE_UART UART_NUM_0

/**
 * Parse and apply one command line. Supported:
 *   setdist <slot-id> <cm>   inject a raw distance for slot's next scan
 */
static void handle_line(const char* line) {
    unsigned slot_id = 0;
    float cm = 0.0f;

    if (sscanf(line, "setdist %u %f", &slot_id, &cm) != 2) {
        ESP_LOGW(TAG, "Unknown or malformed command: %s", line);
        return;
    }

    if (slot_id < 1 || slot_id > PARKING_SLOT_COUNT) {
        ESP_LOGW(TAG, "Slot id %u out of range (1..%d)", slot_id, PARKING_SLOT_COUNT);
        return;
    }

    esp_err_t err = parking_debug_inject_distance((size_t)(slot_id - 1), cm);
    if (err != ESP_OK) {
        ESP_LOGW(TAG, "Inject failed for slot %u: %s", slot_id, esp_err_to_name(err));
    }
}

/**
 * Line-oriented reader: bytes accumulate until CR/LF, then dispatch. The UART
 * driver is installed on the console port so reads can block in this task
 * without disturbing stdout logging.
 */
static void debug_console_task(void* arg) {
    (void)arg;

    if (uart_driver_install(DEBUG_CONSOLE_UART, 1024, 0, 0, NULL, 0) != ESP_OK) {
        ESP_LOGE(TAG, "UART driver install failed; injection disabled");
        vTaskDelete(NULL);
    }

    uint8_t ch;
    char line[64];
    size_t pos = 0;

    while (1) {
        int n = uart_read_bytes(DEBUG_CONSOLE_UART, &ch, 1, pdMS_TO_TICKS(100));
        if (n <= 0) {
            continue;
        }

        if (ch == '\n' || ch == '\r') {
            if (pos > 0) {
                line[pos] = '\0';
                handle_line(line);
                pos = 0;
            }
        } else if (pos < sizeof(line) - 1) {
            line[pos++] = (char)ch;
        }
    }
}

void debug_console_start(void) {
    BaseType_t created =
        xTaskCreate(debug_console_task, "dbg_console", 3072, NULL, 3, NULL);
    ESP_ERROR_CHECK(created == pdPASS ? ESP_OK : ESP_FAIL);
}

#endif /* CONFIG_PARKING_DEBUG_INJECT */
