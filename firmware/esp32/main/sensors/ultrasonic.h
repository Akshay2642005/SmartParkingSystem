#pragma once

#include "driver/gpio.h"
#include "esp_err.h"

/**
 * HC-SR04 ultrasonic sensor driver.
 *
 * Spec: docs/specs/decisions/ADR-0004-sensor-selection.md (sensor selection),
 *       docs/specs/architecture/hardware-architecture.md (GPIO mapping),
 *       docs/specs/product/REQUIREMENTS.md (FR-010, FR-011, FR-012).
 *
 * Measurement limits (HC-SR04 datasheet):
 *   - minimum distance: 2 cm
 *   - maximum distance: 400 cm
 *   - measurement timeout: 30 ms (covers the ~23 ms max round trip at 400 cm)
 *
 * Error policy: all runtime failures (unresponsive sensor, implausible
 * reading, GPIO no longer usable) are recoverable by design and reported as
 * esp_err_t codes — never aborts. The caller decides what to do; the parking
 * layer maps every failure to a slot ERROR state and keeps scanning.
 * Policy: docs/specs/architecture/firmware-architecture.md
 * (§ Error Handling Policy).
 */

/** Minimum measurable distance in centimeters. */
#define ULTRASONIC_MIN_DISTANCE_CM 2.0f

/** Maximum usable distance in centimeters. */
#define ULTRASONIC_MAX_DISTANCE_CM 400.0f

/**
 * Check whether a distance reading is physically plausible for this sensor.
 *
 * Single source of truth for measurement plausibility, shared by the driver
 * and by the parking layer's defense-in-depth validation. Accepts finite,
 * positive readings within [ULTRASONIC_MIN_DISTANCE_CM - tolerance,
 * ULTRASONIC_MAX_DISTANCE_CM]; the tolerance below the datasheet floor
 * absorbs microsecond timing quantization and edge-detection jitter.
 *
 * @param distance_cm Reading to check.
 * @return true if the reading is usable, false if it is garbage or out of range.
 */
bool ultrasonic_distance_is_plausible(float distance_cm);

/** GPIO pins driving a single HC-SR04 sensor. */
typedef struct {
    gpio_num_t trig_gpio; /**< GPIO driving the TRIG input (pulse to start a measurement). */
    gpio_num_t echo_gpio; /**< GPIO reading the ECHO output (pulse width = distance). */
} ultrasonic_config_t;

/** Per-sensor instance state. Owns the pins this sensor is wired to. */
typedef struct {
    ultrasonic_config_t config;
} ultrasonic_sensor_t;

/**
 * Configure the GPIOs for a sensor and set the TRIG line idle-low.
 *
 * Validates the GPIO wiring (both pins exist and are distinct) before touching
 * any hardware. A configuration failure is reported as an error, not a crash.
 *
 * @param sensor  Sensor instance to initialize (filled from @p config).
 * @param config  Pin configuration; must not be NULL.
 * @return ESP_OK on success, ESP_ERR_INVALID_ARG if an argument or a pin is
 *         invalid (out of range or TRIG == ECHO), or an error from the
 *         underlying GPIO driver.
 */
esp_err_t ultrasonic_init(ultrasonic_sensor_t* sensor, const ultrasonic_config_t* config);

/**
 * Perform one distance measurement.
 *
 * Sends a 10 us TRIG pulse and times the ECHO pulse. The returned distance is
 * the one-way distance to the target in centimeters.
 *
 * @param sensor      Initialized sensor instance.
 * @param distance_cm [out] Measured distance; only valid when ESP_OK is returned.
 * @return ESP_OK on success,
 *         ESP_ERR_INVALID_ARG if an argument is invalid,
 *         ESP_ERR_INVALID_STATE if pulsing TRIG fails at runtime (GPIO no
 *                        longer usable),
 *         ESP_ERR_TIMEOUT if no ECHO pulse arrives within the timeout window or
 *                        the distance exceeds ULTRASONIC_MAX_DISTANCE_CM,
 *         ESP_ERR_INVALID_RESPONSE if the reading is implausible
 *                        (clearly below ULTRASONIC_MIN_DISTANCE_CM — a small
 *                        tolerance around the floor absorbs timing jitter —
 *                        or inconsistent).
 */
esp_err_t ultrasonic_measure_cm(const ultrasonic_sensor_t* sensor, float* distance_cm);
