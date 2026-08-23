/*
 * Host-side stand-in for main/sensors/ultrasonic.c.
 *
 * Replaces only the hardware interaction (GPIO timing, TRIG/ECHO pulses);
 * the driver's public contract is mirrored faithfully:
 *   - ultrasonic_init() performs the same argument validation as the real
 *     driver (NULL checks, pin range, distinct pins) without touching GPIOs.
 *   - ultrasonic_distance_is_plausible() IS the production policy, kept in
 *     source-sync with main/sensors/ultrasonic.c::ultrasonic_distance_is_plausible.
 *     Drift here would silently weaken the suite — update both together.
 */

#include "ultrasonic_stub.h"

#include <math.h>
#include <stdbool.h>

#include "driver/gpio.h"
#include "ultrasonic.h"

/* Scripted-result FIFO plus a fallback outcome for unscripted calls.
 * Capacity is generous; if a test ever wraps it, results are consumed in
 * order anyway — the suite would fail loudly on wrong outcomes. */
#define STUB_QUEUE_CAPACITY 64
#define STUB_DEFAULT_DISTANCE_CM 100.0f

typedef struct {
    bool is_error;
    float cm;
    esp_err_t err;
} stub_result_t;

static stub_result_t s_queue[STUB_QUEUE_CAPACITY];
static size_t s_queue_head = 0;
static size_t s_queue_count = 0;
static stub_result_t s_default = {.is_error = false,
    .cm = STUB_DEFAULT_DISTANCE_CM,
    .err = ESP_OK};
static int s_call_count = 0;

bool ultrasonic_distance_is_plausible(float distance_cm) {
    if (!isfinite(distance_cm) || distance_cm <= 0.0f) {
        return false;
    }

    return distance_cm >= ULTRASONIC_MIN_DISTANCE_CM - ULTRASONIC_MIN_TOLERANCE_CM &&
           distance_cm <= ULTRASONIC_MAX_DISTANCE_CM;
}

esp_err_t ultrasonic_init(ultrasonic_sensor_t* sensor, const ultrasonic_config_t* config) {
    if (sensor == NULL || config == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    // Mirrors the real driver: both pins must exist and must be distinct.
    if (config->trig_gpio < 0 || config->trig_gpio >= GPIO_NUM_MAX || config->echo_gpio < 0 ||
        config->echo_gpio >= GPIO_NUM_MAX || config->trig_gpio == config->echo_gpio) {
        return ESP_ERR_INVALID_ARG;
    }

    sensor->config = *config;

    return ESP_OK;
}

esp_err_t ultrasonic_measure_cm(const ultrasonic_sensor_t* sensor, float* distance_cm) {
    if (sensor == NULL || distance_cm == NULL) {
        return ESP_ERR_INVALID_ARG;
    }

    s_call_count++;

    if (s_queue_count > 0) {
        const stub_result_t result = s_queue[s_queue_head];
        s_queue_head = (s_queue_head + 1) % STUB_QUEUE_CAPACITY;
        s_queue_count--;

        if (result.is_error) {
            return result.err;
        }

        *distance_cm = result.cm;

        return ESP_OK;
    }

    if (s_default.is_error) {
        return s_default.err;
    }

    *distance_cm = s_default.cm;

    return ESP_OK;
}

void stub_measure_reset(void) {
    s_queue_head = 0;
    s_queue_count = 0;
    s_default.cm = STUB_DEFAULT_DISTANCE_CM;
    s_default.err = ESP_OK;
    s_default.is_error = false;
    s_call_count = 0;
}

void stub_measure_push_ok(float distance_cm) {
    stub_result_t* slot = &s_queue[(s_queue_head + s_queue_count) % STUB_QUEUE_CAPACITY];
    s_queue_count++;

    slot->is_error = false;
    slot->cm = distance_cm;
}

void stub_measure_push_error(esp_err_t err) {
    stub_result_t* slot = &s_queue[(s_queue_head + s_queue_count) % STUB_QUEUE_CAPACITY];
    s_queue_count++;

    slot->is_error = true;
    slot->err = err;
}

void stub_measure_set_default_ok(float distance_cm) {
    s_default.is_error = false;
    s_default.cm = distance_cm;
}

int stub_measure_call_count(void) {
    return s_call_count;
}

const char* esp_err_to_name(esp_err_t code) {
    switch (code) {
        case ESP_OK:
            return "ESP_OK";
        case ESP_FAIL:
            return "ESP_FAIL";
        case ESP_ERR_NO_MEM:
            return "ESP_ERR_NO_MEM";
        case ESP_ERR_INVALID_ARG:
            return "ESP_ERR_INVALID_ARG";
        case ESP_ERR_INVALID_STATE:
            return "ESP_ERR_INVALID_STATE";
        case ESP_ERR_INVALID_SIZE:
            return "ESP_ERR_INVALID_SIZE";
        case ESP_ERR_NOT_FOUND:
            return "ESP_ERR_NOT_FOUND";
        case ESP_ERR_NOT_SUPPORTED:
            return "ESP_ERR_NOT_SUPPORTED";
        case ESP_ERR_TIMEOUT:
            return "ESP_ERR_TIMEOUT";
        case ESP_ERR_INVALID_RESPONSE:
            return "ESP_ERR_INVALID_RESPONSE";
        default:
            return "UNKNOWN_ERROR";
    }
}
