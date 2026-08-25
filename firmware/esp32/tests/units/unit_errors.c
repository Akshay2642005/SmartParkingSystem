/*
 * Unit: errors — invalid-measurement rejection and scan-path error handling
 * (FR-012: invalid readings -> ERROR, never silently FREE/OCCUPIED).
 *
 * HOST-ONLY: these cases need the scripted ultrasonic stub to force sensor
 * failures, which cannot exist on the real hardware. They run through
 * parking_lot_scan() so validation, error marking, and statistics are
 * exercised together.
 */

#include "parking_selftest.h"
#include "ultrasonic_stub.h"

static parking_lot_t lot;
static parking_slot_t slots[PARKING_SLOT_COUNT];

/* Observer collector for the transition-notification case. */
static parking_transition_t seen[8];
static size_t seen_count;

static void collect_transition(const parking_transition_t* t, void* ctx) {
    (void)ctx;

    if (seen_count < sizeof(seen) / sizeof(seen[0])) {
        seen[seen_count++] = *t;
    }
}

/** Build an n-slot lot from the production config table. */
static void make_lot(size_t n) {
    stub_measure_reset();

    for (size_t i = 0; i < n; i++) {
        slots[i] = make_slot((uint8_t)(i + 1));
    }

    parking_lot_init(&lot, slots, n);
}

int main(void) {
    case_begin("NaN measurement marks ERROR and leaves the slot unbookable");
    make_lot(1);
    stub_measure_push_ok(NAN);
    CHECK(parking_lot_scan(&lot) == ESP_OK);
    CHECK(parking_slot_get_state(&slots[0]) == PARKING_ERROR);
    CHECK(parking_lot_get_error(&lot) == 1);
    CHECK(parking_lot_get_available(&lot) == 0); /* ERROR slots are not bookable */
    CHECK(parking_lot_get_total(&lot) ==
          parking_lot_get_occupied(&lot) + parking_lot_get_available(&lot) +
              parking_lot_get_error(&lot));
    case_end();

    {
        const float invalid[] = {-5.0f, 0.0f, 450.0f};
        const char* names[] = {"negative", "zero", "beyond max range"};
        for (size_t v = 0; v < 3; v++) {
            char label[96];
            snprintf(label, sizeof(label), "%s reading (%.0f cm) is rejected", names[v], (double)invalid[v]);
            case_begin(label);

            make_lot(1);
            stub_measure_push_ok(invalid[v]);
            CHECK(parking_lot_scan(&lot) == ESP_OK);
            CHECK(parking_slot_get_state(&slots[0]) == PARKING_ERROR);
            case_end();
        }
    }

    case_begin("one failing sensor does not abort the scan");
    make_lot(3);
    stub_measure_push_ok(100.0f);             /* slot 1 measures fine */
    stub_measure_push_error(ESP_ERR_TIMEOUT); /* slot 2 fails */
    /* slot 3 falls back to the scripted default (OK at 100 cm). */
    CHECK(parking_lot_scan(&lot) == ESP_OK);
    CHECK(parking_slot_get_state(&slots[0]) == PARKING_FREE);
    CHECK(parking_slot_get_state(&slots[1]) == PARKING_ERROR);
    CHECK(parking_slot_get_state(&slots[2]) == PARKING_FREE);
    CHECK(parking_lot_get_total(&lot) == 3);
    CHECK(parking_lot_get_available(&lot) == 2);
    CHECK(parking_lot_get_error(&lot) == 1);
    case_end();

    case_begin("mark_error is idempotent and preserves the pre-error state");
    parking_slot_t slot = make_slot(1);
    force_occupied(&slot);
    parking_slot_mark_error(&slot);
    parking_slot_mark_error(&slot);
    CHECK(parking_slot_get_state(&slot) == PARKING_ERROR);
    /* A band reading after recovery restores the pre-error state silently. */
    const parking_event_t event = parking_slot_update(&slot, 32.0f);
    CHECK(event.type == PARKING_EVENT_NONE);
    CHECK(parking_slot_get_state(&slot) == PARKING_OCCUPIED);
    case_end();

    case_begin("scan rejects a NULL lot");
    CHECK(parking_lot_scan(NULL) == ESP_ERR_INVALID_ARG);
    case_end();

    case_begin("scan notifies observers of every committed transition");
    {
        seen_count = 0;
        make_lot(2);

        parking_set_event_observer(collect_transition, NULL);

        const float occupied_cm = slots[0].config.occupied_threshold_cm - 5.0f;
        const float free_cm = slots[1].config.free_threshold_cm + 5.0f;

        /* Two scans confirm slot 1 -> OCCUPIED; slot 2 stays FREE (silent).
         * Each scan consumes one reading per slot, in slot order. */
        stub_measure_push_ok(occupied_cm);
        stub_measure_push_ok(free_cm);
        stub_measure_push_ok(occupied_cm);
        stub_measure_push_ok(free_cm);
        CHECK(parking_lot_scan(&lot) == ESP_OK);
        CHECK(parking_lot_scan(&lot) == ESP_OK);
        CHECK(seen_count == 1); /* second confirmation must NOT re-notify */
        CHECK(seen[0].slot_number == 1 && seen[0].state == PARKING_OCCUPIED);

        /* Slot 1 sensor dies: ERROR is a committed transition too. */
        stub_measure_push_error(ESP_ERR_TIMEOUT);
        stub_measure_push_ok(free_cm);
        CHECK(parking_lot_scan(&lot) == ESP_OK);
        CHECK(seen_count == 2);
        CHECK(seen[1].slot_number == 1 && seen[1].state == PARKING_ERROR);

        /* One valid reading recovers immediately to the pre-error state. */
        stub_measure_push_ok(occupied_cm);
        stub_measure_push_ok(free_cm);
        CHECK(parking_lot_scan(&lot) == ESP_OK);
        CHECK(seen_count == 3);
        CHECK(seen[2].slot_number == 1 && seen[2].state == PARKING_OCCUPIED);

        parking_set_event_observer(NULL, NULL);
    }
    case_end();

    return test_summary("unit/errors");
}
