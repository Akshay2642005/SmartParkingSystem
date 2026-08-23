/*
 * Unit: debounce — N-consecutive confirmation sequences (N=2 from
 * parking_settings.h). Each sequence runs against a fresh production slot.
 */

#include "parking_selftest.h"

static void run_sequence(const char* name,
    const float* inputs,
    const parking_event_type_t* expected_events,
    size_t count,
    parking_state_t expected_final_state) {
    case_begin(name);

    parking_slot_t slot = make_slot(1);
    for (size_t i = 0; i < count; i++) {
        const parking_event_t event = parking_slot_update(&slot, inputs[i]);
        if (event.type != expected_events[i]) {
            printf("%s: step %zu (%.1f cm): got event %d, want %d\n",
                paint(COLOR_RED, "[err]"),
                i + 1,
                (double)inputs[i],
                (int)event.type,
                (int)expected_events[i]);
            g_checks_failed++;
        }
    }

    CHECK(parking_slot_get_state(&slot) == expected_final_state);
    case_end();
}

int selftest_debounce(void) {
    {
        const float inputs[] = {25.0f, 25.0f};
        const parking_event_type_t events[] =
            {PARKING_EVENT_NONE, PARKING_EVENT_SLOT_OCCUPIED};
        run_sequence("[25, 25] fires OCCUPIED on the second sighting",
            inputs,
            events,
            2,
            PARKING_OCCUPIED);
    }
    {
        const float inputs[] = {25.0f, 40.0f, 25.0f, 25.0f};
        const parking_event_type_t events[] = {PARKING_EVENT_NONE,
            PARKING_EVENT_NONE,
            PARKING_EVENT_NONE,
            PARKING_EVENT_SLOT_OCCUPIED};
        run_sequence("[25, 40, 25, 25] interleaved opposite reading restarts",
            inputs,
            events,
            4,
            PARKING_OCCUPIED);
    }
    {
        const float inputs[] = {40.0f, 33.0f, 36.0f};
        const parking_event_type_t events[] = {PARKING_EVENT_NONE,
            PARKING_EVENT_NONE,
            PARKING_EVENT_NONE};
        run_sequence("[40, 33, 36] agreement and band never confirm",
            inputs,
            events,
            3,
            PARKING_FREE);
    }
    {
        const float inputs[] = {28.0f, 28.0f, 45.0f, 45.0f};
        const parking_event_type_t events[] = {PARKING_EVENT_NONE,
            PARKING_EVENT_SLOT_OCCUPIED,
            PARKING_EVENT_NONE,
            PARKING_EVENT_SLOT_FREED};
        run_sequence("[28, 28, 45, 45] emits both transitions",
            inputs,
            events,
            4,
            PARKING_FREE);
    }
    {
        const float inputs[] = {20.0f, 50.0f, 20.0f, 50.0f};
        const parking_event_type_t events[] = {PARKING_EVENT_NONE,
            PARKING_EVENT_NONE,
            PARKING_EVENT_NONE,
            PARKING_EVENT_NONE};
        run_sequence("[20, 50, 20, 50] flip-flopping noise never transitions",
            inputs,
            events,
            4,
            PARKING_FREE);
    }

    return test_summary("unit/debounce");
}
