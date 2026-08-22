#pragma once

/*
 * Redis-style micro test framework for the host unit tests.
 *
 * Units print one line per case, mirroring Redis's test output:
 *     [ok]: GEOADD create (1 ms)
 * A case is a group of CHECKs; a failing CHECK prints an [err] line with
 * file/line, the case is marked [FAIL], and the unit exits non-zero.
 *
 * [ok] renders green and [FAIL]/[err] red like Redis's suite — but only on
 * a terminal: colors are disabled when stdout is piped (CI logs) and when
 * NO_COLOR is set (https://no-color.org).
 */

#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

#define COLOR_GREEN "\033[32m"
#define COLOR_RED "\033[31m"
#define COLOR_RESET "\033[0m"

static int g_checks_failed = 0;
static int g_cases_run = 0;
static int g_case_failures_at_start = 0;
static struct timespec g_case_started;
static const char* g_case_name;

static bool use_color(void) {
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
static const char* paint(const char* color, const char* text) {
    static char buffer[160];
    if (use_color()) {
        snprintf(buffer, sizeof(buffer), "%s%s%s", color, text, COLOR_RESET);
    } else {
        snprintf(buffer, sizeof(buffer), "%s", text);
    }
    return buffer;
}

#define CHECK(cond)                        \
    do {                                   \
        if (!(cond)) {                     \
            printf("%s %s:%d: %s\n",       \
                paint(COLOR_RED, "[err]"), \
                __FILE__,                  \
                __LINE__,                  \
                #cond);                    \
            g_checks_failed++;             \
        }                                  \
    } while (0)

/** Float equality within EMA/round-off tolerance. */
#define CHECK_APPROX_EQ(actual, expected) \
    CHECK(fabsf((float)(actual) - (float)(expected)) < 1e-4f)

static void case_begin(const char* name) {
    g_case_name = name;
    g_case_failures_at_start = g_checks_failed;
    clock_gettime(CLOCK_MONOTONIC, &g_case_started);
}

static void case_end(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    const long ms = (now.tv_sec - g_case_started.tv_sec) * 1000L +
                    (now.tv_nsec - g_case_started.tv_nsec) / 1000000L;
    const bool failed = g_checks_failed > g_case_failures_at_start;

    g_cases_run++;
    printf("%s: %s (%ld ms)\n",
        failed ? paint(COLOR_RED, "[FAIL]") : paint(COLOR_GREEN, "[ok]"),
        g_case_name,
        ms);
}

/*
 * Print the failure marker (nothing on success — the caller aggregates) and
 * return the process exit code.
 */
static int test_summary(const char* unit_name) {
    if (g_checks_failed == 0) {
        return 0;
    }

    char message[128];
    snprintf(message,
        sizeof(message),
        "!!! %s FAILED (%d of %d cases had errors)",
        unit_name,
        g_checks_failed,
        g_cases_run);
    printf("\n%s\n", paint(COLOR_RED, message));

    return 1;
}
