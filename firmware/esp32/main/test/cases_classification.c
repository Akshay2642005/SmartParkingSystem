/*
 * Unit: classification — threshold boundaries and hysteresis band semantics.
 *
 * Exercises parking_slot_update() against the production thresholds
 * (occupied <= 30 cm, free >= 35 cm, band (30, 35) ambiguous).
 * Matrix rows: PROMPT.md Phase 20 lines 845-852.
 */

#include "parking_selftest.h"

int selftest_classification(void) {
    parking_slot_t slot = make_slot(1);

    case_begin("fresh slot starts FREE");
    CHECK(parking_slot_get_state(&slot) == PARKING_FREE);
    case_end();

    case_begin("30.0 cm from FREE arms OCCUPIED without an event");
    parking_event_t event = parking_slot_update(&slot, 30.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&slot) == PARKING_FREE);
    case_end();

    case_begin("second occupied-side reading confirms transition with context");
    event = parking_slot_update(&slot, 29.0f);
    CHECK(event.type == PARKING_EVENT_SLOT_OCCUPIED);
    CHECK(event.slot_id == 1);
    CHECK_APPROX_EQ(event.distance_cm, 29.0f);
    CHECK(parking_slot_get_state(&slot) == PARKING_OCCUPIED);
    case_end();

    case_begin("band reading while OCCUPIED holds the state");
    event = parking_slot_update(&slot, 33.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&slot) == PARKING_OCCUPIED);
    case_end();

    case_begin("band reading resets the confirmation counter");
    event = parking_slot_update(&slot, 40.0f); /* free-side sighting #1 */
    CHECK(event.type == PARKING_EVENT_NONE);
    event = parking_slot_update(&slot, 32.0f); /* ambiguous: counter resets */
    CHECK(event.type == PARKING_EVENT_NONE);
    event = parking_slot_update(&slot, 40.0f); /* back to sighting #1 */
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&slot) == PARKING_OCCUPIED);
    event = parking_slot_update(&slot, 36.0f); /* sighting #2 -> freed */
    CHECK(event.type == PARKING_EVENT_SLOT_FREED);
    CHECK(parking_slot_get_state(&slot) == PARKING_FREE);
    case_end();

    case_begin("34.0 cm stays band, 35.0 cm is free-side");
    parking_slot_t edge = make_slot(2);
    force_occupied(&edge);
    event = parking_slot_update(&edge, 34.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&edge) == PARKING_OCCUPIED);
    event = parking_slot_update(&edge, 35.0f); /* free-side sighting #1 */
    CHECK(event.type == PARKING_EVENT_NONE);
    event = parking_slot_update(&edge, 35.0f); /* sighting #2 -> freed */
    CHECK(event.type == PARKING_EVENT_SLOT_FREED);
    CHECK(parking_slot_get_state(&edge) == PARKING_FREE);
    case_end();

    case_begin("repeated band readings never occupy a FREE slot");
    parking_slot_t held = make_slot(3);
    event = parking_slot_update(&held, 31.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    event = parking_slot_update(&held, 32.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    event = parking_slot_update(&held, 31.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&held) == PARKING_FREE);
    case_end();

    return test_summary("unit/classification");
}
