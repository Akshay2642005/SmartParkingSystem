#pragma once

/*
 * Portable  micro test framework + fixture for the parking domain.
 *
 * The SAME case sources run in two places:
 *
 *   - Host unit tests (firmware/esp32/tests/host): compiled against stubs of
 *     the IDF surface by a standalone CMake project, driven by `make test`.
 *   - On-target self-test: linked into the real firmware behind
 *     CONFIG_PARKING_DEBUG_INJECT and triggered with the `selftest` serial
 *     command, so the production binary validates itself under FreeRTOS/IDF
 *     (and headlessly inside Wokwi).
 *
 * Output mirrors Redis's suite, one line per case:
 *     [ok]: GEOADD create (1 ms)
 * A failing CHECK prints an [err] line with file/line, marks the case [FAIL],
 * and makes the enclosing unit return non-zero.
 *
 * Colors ([ok] green, [FAIL]/[err] red) are terminal-only on the host; the
 * target build never colors (the UART console is not a color terminal).
 */

#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef PARKING_SELFTEST_ON_TARGET
#include "esp_timer.h"
#else
#include <time.h>
#include <unistd.h>
#endif

#include "parking.h"
#include "parking_config.h"

#define COLOR_GREEN "\033[32m"
#define COLOR_RED "\033[31m"
#define COLOR_RESET "\033[0m"

__attribute__((unused)) static int g_checks_failed = 0;
__attribute__((unused)) static int g_case_failures_at_start = 0;
static const char* g_case_name;

#ifndef PARKING_SELFTEST_ON_TARGET

__attribute__((unused)) static struct timespec g_case_started;

__attribute__((unused)) static bool use_color(void) {
    static int cached = -1;
    if (cached == -1) {
        cached = (isatty(fileno(stdout)) && getenv("NO_COLOR") == NULL) ? 1 : 0;
    }
    return cached;
}

/*
 * Colored label helper. Returns a pointer to a shared static buffer — use
 * at most ONE paint() call per printf statement.
 */
__attribute__((unused)) static const char* paint(const char* color, const char* text) {
    static char buffer[160];
    if (use_color()) {
        snprintf(buffer, sizeof(buffer), "%s%s%s", color, text, COLOR_RESET);
    } else {
        snprintf(buffer, sizeof(buffer), "%s", text);
    }
    return buffer;
}

#else /* target: serial console, never colored */

__attribute__((unused)) static const char* paint(const char* color, const char* text) {
    (void)color;

    return text;
}

#endif

#ifdef PARKING_SELFTEST_ON_TARGET

__attribute__((unused)) static long timer_now_ms(void) {
    return (long)(esp_timer_get_time() / 1000);
}

#else

__attribute__((unused)) static struct timespec g_case_started;

__attribute__((unused)) static long timer_now_ms(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);

    return (long)(now.tv_sec * 1000L + now.tv_nsec / 1000000L);
}

#endif

__attribute__((unused)) static long g_case_started_ms;

#define CHECK(cond)                  \
    do {                             \
        if (!(cond)) {               \
            printf("%s %s:%d: %s\n", \
                paint(COLOR_RED,     \
                    "[err]"),        \
                __FILE__,            \
                __LINE__,            \
                #cond);              \
            g_checks_failed++;       \
        }                            \
    } while (0)

/** Float equality within EMA/round-off tolerance. */
#define CHECK_APPROX_EQ(actual, expected) \
    CHECK(fabsf((float)(actual) - (float)(expected)) < 1e-4f)

__attribute__((unused)) static void case_begin(const char* name) {
    g_case_name = name;
    g_case_failures_at_start = g_checks_failed;
    g_case_started_ms = timer_now_ms();
}

__attribute__((unused)) static void case_end(void) {
    const long ms = timer_now_ms() - g_case_started_ms;
    const bool failed = g_checks_failed > g_case_failures_at_start;

    printf("%s: %s (%ld ms)\n",
        failed ? paint(COLOR_RED, "[FAIL]") : paint(COLOR_GREEN, "[ok]"),
        g_case_name,
        ms);
}

/*
 * Print the failure marker (nothing on success — the caller aggregates) and
 * return the process exit code.
 */
__attribute__((unused)) static int test_summary(const char* unit_name) {
    if (g_checks_failed == 0) {
        return 0;
    }

    printf("\n!!! %s FAILED (%d checks)\n", unit_name, g_checks_failed);

    return 1;
}

/* --- fixture -------------------------------------------------------------- */

/* Abort the whole unit on init failure — configuration errors are build
 * problems, not test cases. */
__attribute__((unused)) static parking_slot_t make_slot(uint8_t id) {
    parking_slot_t slot;
    const esp_err_t err = parking_slot_init(&slot, &slot_configs[id - 1]);

    if (err != ESP_OK) {
        fprintf(stderr, "FATAL: parking_slot_init(id=%u) -> %d\n", id, err);
        exit(2);
    }

    return slot;
}

/** Drive a fresh FREE slot into OCCUPIED via two confirmed readings (N=2). */
__attribute__((unused)) static inline void force_occupied(parking_slot_t* slot) {
    const float occupied_cm = slot->config.occupied_threshold_cm - 5.0f;
    (void)parking_slot_update(slot, occupied_cm);
    (void)parking_slot_update(slot, occupied_cm);
}

/** Drive an OCCUPIED slot back to FREE via two confirmed readings. */
__attribute__((unused)) static inline void force_free(parking_slot_t* slot) {
    const float free_cm = slot->config.free_threshold_cm + 5.0f;
    (void)parking_slot_update(slot, free_cm);
    (void)parking_slot_update(slot, free_cm);
}

/* --- portable units (compiled into firmware AND host tests) --------------- */

int selftest_classification(void);
int selftest_debounce(void);
int selftest_recovery(void);
int selftest_statistics(void);
