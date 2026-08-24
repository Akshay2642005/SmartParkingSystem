/* Host wrapper: the cases live in main/test/cases_payload.c so the same
 * checks also run on-target via the `selftest` serial command. */

#include "parking_selftest.h"

int main(void) {
    return selftest_payload();
}
