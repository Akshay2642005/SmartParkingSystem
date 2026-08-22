#include "ultrasonic.h"

#include <math.h>

#include "esp_log.h"
#include "esp_rom_sys.h"
#include "esp_timer.h"

/**
 * HC-SR04 ultrasonic sensor driver.
 *
 * Spec: docs/specs/decisions/ADR-0004-sensor-selection.md,
 *       docs/specs/product/REQUIREMENTS.md (FR-010, FR-011, FR-012).
 *
 * A single measurement is:
 *   1. a 10 us HIGH pulse on TRIG (starts one measurement cycle),
 *   2. wait for ECHO to go HIGH (measurement in progress),
 *   3. wait for ECHO to go LOW (the pulse width is the round-trip time),
 *   4. convert the round-trip time to a one-way distance.
 */

static const char* TAG = "ultrasonic";

/** Minimum measurable distance in cm (HC-SR04 datasheet). */
#define ULTRASONIC_MIN_DISTANCE_CM 2.0f

/**
 * Tolerance applied to the minimum-distance cutoff, in cm.
 *
 * The round trip at the datasheet floor is only ~116 us, so microsecond
 * quantization of the ECHO pulse and edge-detection jitter can shave a
 * legitimate near-floor reading just below ULTRASONIC_MIN_DISTANCE_CM.
 * Readings within this tolerance are kept; anything shorter is a spurious echo.
 */
#define ULTRASONIC_MIN_TOLERANCE_CM 0.5f

/** Maximum usable distance in cm (HC-SR04 datasheet). */
#define ULTRASONIC_MAX_DISTANCE_CM 400.0f

/** Max time (us) to wait for ECHO to rise and to fall again. */
#define ULTRASONIC_TIMEOUT_US 30000

/** Speed of sound in cm per microsecond (round trip is halved in the formula). */
#define SOUND_SPEED_CM_PER_US 0.0343f

/** Width (us) of the HIGH pulse on TRIG that starts a measurement. */
#define TRIG_PULSE_US 10

/** Idle settling time (us) between measurements. */
#define TRIG_IDLE_US 2

esp_err_t ultrasonic_init(ultrasonic_sensor_t* sensor, const ultrasonic_config_t* config) {
    if (sensor == NULL || config == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    // Validate GPIO wiring before touching hardware: both pins must exist on the
    // target and must be distinct. Prevents out-of-range shifts on pin_bit_mask.
    if (config->trig_gpio < 0 || config->trig_gpio >= GPIO_NUM_MAX || config->echo_gpio < 0 ||
        config->echo_gpio >= GPIO_NUM_MAX || config->trig_gpio == config->echo_gpio) {
        return ESP_ERR_INVALID_ARG;
    }

    sensor->config = *config;

    gpio_config_t trig_config = {
        .pin_bit_mask = 1ULL << config->trig_gpio,
        .mode = GPIO_MODE_OUTPUT,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };

    // A failing gpio_config is a configuration error, not a fatal one: propagate
    // the error so the caller decides whether to retry or give up.
    esp_err_t err = gpio_config(&trig_config);
    if (err != ESP_OK) {
        return err;
    }

    gpio_config_t echo_config = {
        .pin_bit_mask = 1ULL << config->echo_gpio,
        .mode = GPIO_MODE_INPUT,
        .pull_up_en = GPIO_PULLUP_DISABLE,
        .pull_down_en = GPIO_PULLDOWN_DISABLE,
        .intr_type = GPIO_INTR_DISABLE,
    };

    err = gpio_config(&echo_config);
    if (err != ESP_OK) {
        return err;
    }

    // Hold TRIG low so a measurement is only started by an explicit pulse below.
    err = gpio_set_level(config->trig_gpio, 0);
    if (err != ESP_OK) {
        return err;
    }

    ESP_LOGI(TAG, "Initialized TRIG=%d ECHO=%d", config->trig_gpio, config->echo_gpio);

    return ESP_OK;
}

bool ultrasonic_distance_is_plausible(float distance_cm) {
    if (!isfinite(distance_cm) || distance_cm <= 0.0f) {
        return false;
    }

    return distance_cm >= ULTRASONIC_MIN_DISTANCE_CM - ULTRASONIC_MIN_TOLERANCE_CM &&
           distance_cm <= ULTRASONIC_MAX_DISTANCE_CM;
}

esp_err_t ultrasonic_measure_cm(const ultrasonic_sensor_t* sensor, float* distance_cm) {
    if (sensor == NULL || distance_cm == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    const gpio_num_t trig = sensor->config.trig_gpio;
    const gpio_num_t echo = sensor->config.echo_gpio;

    // A 10 us HIGH pulse on TRIG starts one measurement cycle. A GPIO failure
    // here is a runtime condition (pin reconfigured or invalid): report it as
    // a recoverable error instead of measuring garbage.
    esp_err_t trig_result = gpio_set_level(trig, 0);

    if (trig_result == ESP_OK) {
        esp_rom_delay_us(TRIG_IDLE_US);
        trig_result = gpio_set_level(trig, 1);

        if (trig_result == ESP_OK) {
            esp_rom_delay_us(TRIG_PULSE_US);
            trig_result = gpio_set_level(trig, 0);
        }
    }

    if (trig_result != ESP_OK) {
        // Runtime GPIO failure -> recoverable error, not a crash.
        return ESP_ERR_INVALID_STATE;
    }

    // Wait for ECHO to go HIGH (measurement in progress).
    int64_t timeout_start = esp_timer_get_time();

    while (gpio_get_level(echo) == 0) {
        if (esp_timer_get_time() - timeout_start >= ULTRASONIC_TIMEOUT_US) {
            // No object within range, or the sensor is unresponsive.
            return ESP_ERR_TIMEOUT;
        }
    }

    int64_t pulse_start = esp_timer_get_time();

    // Wait for ECHO to go LOW (measurement complete). Pulse width is the round trip.
    while (gpio_get_level(echo) == 1) {
        if (esp_timer_get_time() - pulse_start >= ULTRASONIC_TIMEOUT_US) {
            return ESP_ERR_TIMEOUT;
        }
    }

    int64_t pulse_end = esp_timer_get_time();

    const int64_t pulse_duration_us = pulse_end - pulse_start;

    if (pulse_duration_us < 0) {
        // Defensive: pulse_end must not precede pulse_start.
        return ESP_ERR_INVALID_RESPONSE;
    }

    // One-way distance = (round-trip time * sound speed) / 2.
    const float distance = (pulse_duration_us * SOUND_SPEED_CM_PER_US) / 2.0f;

    // Reject implausible readings instead of propagating garbage upstream.
    if (!ultrasonic_distance_is_plausible(distance)) {
        if (distance > ULTRASONIC_MAX_DISTANCE_CM) {
            // Beyond the usable range: treat like "no object in range".
            return ESP_ERR_TIMEOUT;
        }

        // Too close to be a real target: treat as a spurious echo.
        return ESP_ERR_INVALID_RESPONSE;
    }

    *distance_cm = distance;

    return ESP_OK;
}
