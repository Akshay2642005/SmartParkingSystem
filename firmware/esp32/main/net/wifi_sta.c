#include "wifi_sta.h"

#include <string.h>

#include "esp_event.h"
#include "esp_log.h"
#include "esp_netif.h"
#include "esp_wifi.h"
#include "freertos/FreeRTOS.h"
#include "freertos/semphr.h"
#include "nvs_flash.h"

static const char* TAG = "wifi";

/** Connect window before giving up; the caller owns the retry policy. */
#define WIFI_CONNECT_TIMEOUT_MS 20000

static SemaphoreHandle_t s_got_ip;

/* One-time heavy init; guarded so a retry after timeout does not redo it. */
static bool s_stack_ready;

static void wifi_event_handler(void* arg,
    esp_event_base_t event_base,
    int32_t event_id,
    void* event_data) {
    (void)arg;
    (void)event_data;

    if (event_base == WIFI_EVENT) {
        if (event_id == WIFI_EVENT_STA_START) {
            // IDF never joins implicitly - starting the radio only brings up
            // the interface. The association must be requested explicitly
            // (matches examples/wifi/getting_started/station).
            esp_wifi_connect();
        }
        // Disconnection handling stays with the caller's retry loop, which
        // re-runs the full stop -> configure -> start -> connect sequence.
        return;
    }

    if (event_id == IP_EVENT_STA_GOT_IP) {
        const ip_event_got_ip_t* event = (const ip_event_got_ip_t*)event_data;
        ESP_LOGI(TAG, "Got IP: " IPSTR, IP2STR(&event->ip_info.ip));
        xSemaphoreGive(s_got_ip);
    }
}

esp_err_t wifi_sta_connect(void) {
    if (!s_stack_ready) {
        // NVS is required by esp_wifi; erase-and-retry covers the
        // first-boot / upgrade cases where pages come up invalid.
        esp_err_t err = nvs_flash_init();
        if (err == ESP_ERR_NVS_NO_FREE_PAGES || err == ESP_ERR_NVS_NEW_VERSION_FOUND) {
            ESP_ERROR_CHECK(nvs_flash_erase());
            ESP_ERROR_CHECK(nvs_flash_init());
        }

        ESP_ERROR_CHECK(esp_netif_init());
        ESP_ERROR_CHECK(esp_event_loop_create_default());

        // Returns the station netif handle (NULL on failure) - NOT an
        // esp_err_t; never feed it to ESP_ERROR_CHECK.
        const esp_netif_t* sta_netif = esp_netif_create_default_wifi_sta();
        ESP_ERROR_CHECK(sta_netif == NULL ? ESP_FAIL : ESP_OK);

        const wifi_init_config_t cfg = WIFI_INIT_CONFIG_DEFAULT();
        ESP_ERROR_CHECK(esp_wifi_init(&cfg));

        s_got_ip = xSemaphoreCreateBinary();
        ESP_ERROR_CHECK(esp_event_handler_register(
            WIFI_EVENT, ESP_EVENT_ANY_ID, wifi_event_handler, NULL));
        ESP_ERROR_CHECK(esp_event_handler_register(
            IP_EVENT, IP_EVENT_STA_GOT_IP, wifi_event_handler, NULL));

        s_stack_ready = true;
    }

    wifi_config_t wifi_config = {0};
    strlcpy((char*)wifi_config.sta.ssid, CONFIG_PARKING_WIFI_SSID, sizeof(wifi_config.sta.ssid));
    strlcpy((char*)wifi_config.sta.password, CONFIG_PARKING_WIFI_PASSWORD, sizeof(wifi_config.sta.password));

    if (wifi_config.sta.password[0] == '\0') {
        wifi_config.sta.threshold.authmode = WIFI_AUTH_OPEN;
    } else {
        wifi_config.sta.threshold.authmode = WIFI_AUTH_WPA_WPA2_PSK;
    }

    // Reset any half-open join from a previous timed-out attempt so the
    // sequence below is safe to repeat.
    (void)esp_wifi_disconnect();
    (void)esp_wifi_stop();

    ESP_ERROR_CHECK(esp_wifi_set_mode(WIFI_MODE_STA));
    ESP_ERROR_CHECK(esp_wifi_set_config(WIFI_IF_STA, &wifi_config));
    ESP_ERROR_CHECK(esp_wifi_start());

    if (xSemaphoreTake(s_got_ip, pdMS_TO_TICKS(WIFI_CONNECT_TIMEOUT_MS)) != pdTRUE) {
        ESP_LOGE(TAG,
            "No IP within %d ms (SSID \"%s\")",
            WIFI_CONNECT_TIMEOUT_MS,
            CONFIG_PARKING_WIFI_SSID);

        return ESP_ERR_TIMEOUT;
    }

    return ESP_OK;
}
