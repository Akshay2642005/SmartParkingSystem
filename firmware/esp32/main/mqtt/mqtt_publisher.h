#pragma once

#include "esp_err.h"

/**
 * Wire the telemetry path together and start the MQTT publisher task.
 *
 * Registers the parking-domain transition observer (so the task never reads
 * domain state cross-task) and creates the mqtt_task, which connects WiFi,
 * then the broker. Call exactly once during startup, before the parking task
 * begins scanning; returns immediately after task creation.
 *
 * Only exists when CONFIG_PARKING_MQTT_ENABLE is set.
 *
 * @return ESP_OK on success, ESP_FAIL if the task could not be created.
 */
esp_err_t parking_mqtt_publisher_start(void);
