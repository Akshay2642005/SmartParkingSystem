#pragma once

/*
 * Scripting API for the host-side ultrasonic driver stand-in
 * (see stubs/ultrasonic_stub.c).
 *
 * Tests queue results that ultrasonic_measure_cm() consumes in FIFO order;
 * once the queue is empty every call returns the default success value.
 * This lets scan-path tests script per-slot outcomes deterministically,
 * because parking_lot_scan() measures slots in index order.
 */

#include "esp_err.h"

/** Clear queued results and restore the default outcome (OK at 100 cm). */
void stub_measure_reset(void);

/** Queue one successful measurement returning @p distance_cm. */
void stub_measure_push_ok(float distance_cm);

/** Queue one failed measurement returning @p err. */
void stub_measure_push_error(esp_err_t err);

/** Change the fallback outcome used when the queue is empty. */
void stub_measure_set_default_ok(float distance_cm);

/** @return Number of ultrasonic_measure_cm() calls since the last reset. */
int stub_measure_call_count(void);
