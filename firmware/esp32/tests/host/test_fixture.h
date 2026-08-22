#pragma once

/*
 * Shared slot fixture for the host units.
 *
 * Slots are built from the PRODUCTION configuration table (real GPIOs and
 * real thresholds: occupied <= 30 cm, free >= 35 cm), so every assertion in
 * the units exercises the numbers that ship.
 */

#include <stdio.h>
#include <stdlib.h>

#include "parking.h"
#include "parking_config.h"

/* Abort the whole unit on init failure — configuration errors are build
 * problems, not test cases. */
static parking_slot_t make_slot(uint8_t id) {
    parking_slot_t slot;
    const esp_err_t err = parking_slot_init(&slot, &slot_configs[id - 1]);

    if (err != ESP_OK) {
        fprintf(stderr, "FATAL: parking_slot_init(id=%u) -> %d\n", id, err);
        exit(2);
    }

    return slot;
}

/** Drive a fresh FREE slot into OCCUPIED via two confirmed readings (N=2). */
static inline void force_occupied(parking_slot_t* slot) {
    const float occupied_cm = slot->config.occupied_threshold_cm - 5.0f;
    (void)parking_slot_update(slot, occupied_cm);
    (void)parking_slot_update(slot, occupied_cm);
}

/** Drive an OCCUPIED slot back to FREE via two confirmed readings. */
static inline void force_free(parking_slot_t* slot) {
    const float free_cm = slot->config.free_threshold_cm + 5.0f;
    (void)parking_slot_update(slot, free_cm);
    (void)parking_slot_update(slot, free_cm);
}
