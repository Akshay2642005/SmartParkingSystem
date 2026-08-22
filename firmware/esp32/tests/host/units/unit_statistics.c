/*
 * Unit: statistics — lot counters, the total = occupied + available + error
 * invariant, and the query API's documented NULL conventions.
 */

#include <stdio.h>
#include <string.h>

#include "test_fixture.h"
#include "test_util.h"

int main(void) {
    case_begin("mixed states produce correct counters");
    parking_lot_t lot;
    parking_slot_t slots[3];

    for (size_t i = 0; i < 3; i++) {
        slots[i] = make_slot((uint8_t)(i + 1));
    }
    parking_lot_init(&lot, slots, 3);

    force_occupied(&slots[0]);          /* occupied */
    parking_slot_mark_error(&slots[2]); /* error; slot 1 stays FREE */

    parking_lot_update_counts(&lot);

    CHECK(parking_lot_get_total(&lot) == 3);
    CHECK(parking_lot_get_occupied(&lot) == 1);
    CHECK(parking_lot_get_available(&lot) == 1); /* ERROR excluded */
    CHECK(parking_lot_get_error(&lot) == 1);
    CHECK(parking_lot_get_total(&lot) ==
          parking_lot_get_occupied(&lot) + parking_lot_get_available(&lot) +
              parking_lot_get_error(&lot));
    case_end();

    case_begin("counts track later transitions");
    force_occupied(&slots[1]); /* second slot fills up */
    parking_lot_update_counts(&lot);
    CHECK(parking_lot_get_occupied(&lot) == 2);
    CHECK(parking_lot_get_available(&lot) == 0);
    CHECK(parking_lot_get_error(&lot) == 1);
    case_end();

    case_begin("NULL conventions read as unknown/error");
    CHECK(parking_lot_get_total(NULL) == 0);
    CHECK(parking_lot_get_occupied(NULL) == 0);
    CHECK(parking_lot_get_available(NULL) == 0);
    CHECK(parking_lot_get_error(NULL) == 0);
    CHECK(parking_slot_get_state(NULL) == PARKING_ERROR);
    CHECK_APPROX_EQ(parking_slot_get_distance_cm(NULL), 0.0f);
    case_end();

    case_begin("state names match the serial vocabulary");
    CHECK(strcmp(parking_state_to_string(PARKING_FREE), "FREE") == 0);
    CHECK(strcmp(parking_state_to_string(PARKING_OCCUPIED), "OCCUPIED") == 0);
    CHECK(strcmp(parking_state_to_string(PARKING_ERROR), "ERROR") == 0);
    case_end();

    return test_summary("unit/statistics");
}
