#pragma once

#include <stdbool.h>

#include "esp_err.h"

/**
 * Bring up the WiFi station stack and join the configured network
 * (CONFIG_PARKING_WIFI_SSID / _PASSWORD), blocking until an IP is obtained
 * or WIFI_CONNECT_TIMEOUT_MS elapses.
 *
 * Heavy stack initialization happens once; subsequent calls only re-apply
 * credentials and restart the station, so callers may retry after failure.
 *
 * @return ESP_OK once an IP is acquired, ESP_ERR_TIMEOUT on connect timeout,
 *         or any propagated init error.
 */
esp_err_t wifi_sta_connect(void);
