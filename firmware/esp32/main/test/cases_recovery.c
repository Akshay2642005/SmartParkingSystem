/*
 * Unit: recovery — deterministic ERROR recovery semantics from ADR-0007:
 * the first valid measurement decides (deliberately not debounced), an
 * ambiguous band reading restores state_before_error, and only a genuine
 * occupancy change emits an event.
 */

#include "parking_selftest.h"

int selftest_recovery(void) {
    case_begin("band recovery restores pre-error OCCUPIED silently");
    parking_slot_t slot = make_slot(1);
    force_occupied(&slot);
    parking_slot_mark_error(&slot);
    CHECK(parking_slot_get_state(&slot) == PARKING_ERROR);
    const parking_event_t band_event = parking_slot_update(&slot, 32.0f);
    CHECK(band_event.type == PARKING_EVENT_NONE); /* silent restore */
    CHECK(parking_slot_get_state(&slot) == PARKING_OCCUPIED);
    case_end();

    case_begin("pre-error OCCUPIED plus free-side reading frees with event");
    parking_slot_t was_occupied = make_slot(2);
    force_occupied(&was_occupied);
    parking_slot_mark_error(&was_occupied);
    /* Recovery is immediate: a single valid reading decides. */
    const parking_event_t freed = parking_slot_update(&was_occupied, 40.0f);
    CHECK(freed.type == PARKING_EVENT_SLOT_FREED);
    CHECK(freed.slot_id == 2);
    CHECK_APPROX_EQ(freed.distance_cm, 40.0f);
    CHECK(parking_slot_get_state(&was_occupied) == PARKING_FREE);
    case_end();

    case_begin("pre-error FREE plus occupied-side reading occupies with event");
    parking_slot_t fresh = make_slot(3);
    parking_slot_mark_error(&fresh);
    const parking_event_t occupied =
        parking_slot_update(&fresh, fresh.config.occupied_threshold_cm - 5.0f);
    CHECK(occupied.type == PARKING_EVENT_SLOT_OCCUPIED);
    CHECK(parking_slot_get_state(&fresh) == PARKING_OCCUPIED);
    case_end();

    case_begin("recovered slot resumes normal debounced operation");
    parking_slot_t resumed = make_slot(1);
    force_free(&resumed);
    parking_slot_mark_error(&resumed);
    (void)parking_slot_update(&resumed, 32.0f); /* silent restore to FREE */
    /* After recovery, transitions require confirmation again: one occupied-
     * side reading may not flip the slot on its own. */
    const parking_event_t armed = parking_slot_update(&resumed, 25.0f);
    CHECK(armed.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&resumed) == PARKING_FREE);
    const parking_event_t confirmed = parking_slot_update(&resumed, 25.0f);
    CHECK(confirmed.type == PARKING_EVENT_SLOT_OCCUPIED);
    case_end();

    return test_summary("unit/recovery");
}
