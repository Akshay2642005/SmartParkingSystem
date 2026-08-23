#pragma once

/*
 * Host-test stub of ESP-IDF driver/gpio.h (subset used by sensors/ultrasonic.h).
 *
 * Only the pieces the parking domain touches are modeled: the gpio_num_t
 * value space (including GPIO_NUM_MAX, which the driver uses for pin-range
 * validation) and declarations for the configuration calls. The declarations
 * have no definitions here on purpose: if production code ever starts
 * calling them from a host-compiled translation unit, the link fails loudly
 * instead of silently no-oping.
 *
 * Synced from ESP-IDF v6.0.2 (driver/gpio.h); keep the value ranges aligned.
 */

#include <stdint.h>

#include "esp_err.h"

typedef enum {
    GPIO_NUM_NC = -1, /**< Signals "unconfigured". */
    GPIO_NUM_0 = 0,   /**< GPIO0 */
    GPIO_NUM_1,       /**< GPIO1 */
    GPIO_NUM_2,       /**< GPIO2 */
    GPIO_NUM_3,       /**< GPIO3 */
    GPIO_NUM_4,       /**< GPIO4 */
    GPIO_NUM_5,       /**< GPIO5 */
    GPIO_NUM_6,       /**< GPIO6 */
    GPIO_NUM_7,       /**< GPIO7 */
    GPIO_NUM_8,       /**< GPIO8 */
    GPIO_NUM_9,       /**< GPIO9 */
    GPIO_NUM_10,      /**< GPIO10 */
    GPIO_NUM_11,      /**< GPIO11 */
    GPIO_NUM_12,      /**< GPIO12 */
    GPIO_NUM_13,      /**< GPIO13 */
    GPIO_NUM_14,      /**< GPIO14 */
    GPIO_NUM_15,      /**< GPIO15 */
    GPIO_NUM_16,      /**< GPIO16 */
    GPIO_NUM_17,      /**< GPIO17 */
    GPIO_NUM_18,      /**< GPIO18 */
    GPIO_NUM_19,      /**< GPIO19 */
    GPIO_NUM_20,      /**< GPIO20 */
    GPIO_NUM_21,      /**< GPIO21 */
    GPIO_NUM_22,      /**< GPIO22 */
    GPIO_NUM_23,      /**< GPIO23 */
    GPIO_NUM_24,      /**< GPIO24 */
    GPIO_NUM_25,      /**< GPIO25 */
    GPIO_NUM_26,      /**< GPIO26 */
    GPIO_NUM_27,      /**< GPIO27 */
    GPIO_NUM_28,      /**< GPIO28 */
    GPIO_NUM_29,      /**< GPIO29 */
    GPIO_NUM_30,      /**< GPIO30 */
    GPIO_NUM_31,      /**< GPIO31 */
    GPIO_NUM_32,      /**< GPIO32 */
    GPIO_NUM_33,      /**< GPIO33 */
    GPIO_NUM_34,      /**< GPIO34 */
    GPIO_NUM_35,      /**< GPIO35 */
    GPIO_NUM_36,      /**< GPIO36 */
    GPIO_NUM_37,      /**< GPIO37 */
    GPIO_NUM_38,      /**< GPIO38 */
    GPIO_NUM_39,      /**< GPIO39 */
    GPIO_NUM_MAX      /**< One past the highest valid GPIO number. */
} gpio_num_t;

typedef enum {
    GPIO_INTR_DISABLE = 0 /**< Disable GPIO interrupt (only value we model). */
} gpio_int_type_t;

typedef enum {
    GPIO_MODE_INPUT = 0, /**< Input only. */
    GPIO_MODE_OUTPUT     /**< Output only. */
} gpio_mode_t;

typedef enum {
    GPIO_PULLUP_DISABLE = 0x0, /**< Disable pull-up. */
    GPIO_PULLUP_ENABLE = 0x1   /**< Enable pull-up. */
} gpio_pullup_t;

typedef enum {
    GPIO_PULLDOWN_DISABLE = 0x0, /**< Disable pull-down. */
    GPIO_PULLDOWN_ENABLE = 0x1   /**< Enable pull-down. */
} gpio_pulldown_t;

/** Configuration parameter of gpio_config(). */
typedef struct {
    uint32_t pin_bit_mask;   /**< Bitmask of pins to configure. */
    gpio_mode_t mode;        /**< Pin mode. */
    gpio_pullup_t pull_up_en;   /**< Pull-up setting. */
    gpio_pulldown_t pull_down_en; /**< Pull-down setting. */
    gpio_int_type_t intr_type;    /**< Interrupt type. */
} gpio_config_t;

esp_err_t gpio_config(const gpio_config_t* config);
esp_err_t gpio_set_level(gpio_num_t gpio, uint32_t level);
