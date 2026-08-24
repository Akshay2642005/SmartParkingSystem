/* Protocol v1 payload cases — shared by host suite and on-target selftest. */

#include "parking_payload.h"
#include "parking_selftest.h"
#include <string.h>

static void fill_four_slots(parking_payload_slot_t* out) {
    static const parking_payload_slot_t template[] = {
        {1, PARKING_OCCUPIED, 128450},
        {2, PARKING_FREE, 90000},
        {3, PARKING_ERROR, 777},
        {4, PARKING_FREE, 64},
    };

    for (size_t i = 0; i < 4; ++i) {
        out[i] = template[i];
    }
}

int selftest_payload(void) {
    char buf[320];

    case_begin("golden snapshot matches the wire contract");
    {
        parking_payload_slot_t slots[4];
        fill_four_slots(slots);

        const int n = build_parking_payload("A", slots, 4, 4711, 128456, buf, sizeof(buf));
        CHECK(n > 0);
        /* Byte-exact against docs/specs/architecture/communication.md. */
        CHECK(strcmp(buf,
                  "{\"v\":1,\"ts_ms\":128456,\"seq\":4711,\"section\":\"A\","
                  "\"slots\":["
                  "{\"id\":\"A-1\",\"state\":\"occupied\",\"changed_ms\":128450},"
                  "{\"id\":\"A-2\",\"state\":\"free\",\"changed_ms\":90000},"
                  "{\"id\":\"A-3\",\"state\":\"error\",\"changed_ms\":777},"
                  "{\"id\":\"A-4\",\"state\":\"free\",\"changed_ms\":64}]}") == 0);
        CHECK((size_t)n == strlen(buf));
    }
    case_end();

    case_begin("wire vocabulary is lowercase protocol tokens");
    {
        static const struct {
            parking_state_t state;
            const char* token;
        } table[] = {
            {PARKING_FREE, "free"},
            {PARKING_OCCUPIED, "occupied"},
            {PARKING_ERROR, "error"},
        };

        for (size_t i = 0; i < sizeof(table) / sizeof(table[0]); ++i) {
            const parking_payload_slot_t slot = {1, table[i].state, 0};
            char small[160];
            const int n =
                build_parking_payload("B", &slot, 1, 1, 1, small, sizeof(small));
            CHECK(n > 0);

            char needle[48];
            snprintf(needle, sizeof(needle), "\"state\":\"%s\"", table[i].token);
            CHECK(strstr(small, needle) != NULL);
            /* Serial vocabulary stays uppercase — translation is intentional. */
            CHECK(strcmp(table[i].token, parking_state_to_string(table[i].state)) != 0);
        }
    }
    case_end();

    case_begin("slot ids carry the section prefix");
    {
        const parking_payload_slot_t slot = {12, PARKING_FREE, 5};
        char small[96];
        CHECK(build_parking_payload("C", &slot, 1, 1, 1, small, sizeof(small)) > 0);
        CHECK(strstr(small, "\"id\":\"C-12\"") != NULL);
    }
    case_end();

    case_begin("truncation returns -1");
    {
        parking_payload_slot_t slots[4];
        fill_four_slots(slots);

        char exact[512];
        const int needed =
            build_parking_payload("A", slots, 4, 4711, 128456, exact, sizeof(exact));
        CHECK(needed > 0);

        /* Exactly enough for the string but not its NUL terminator. */
        char tight[(size_t)needed];
        CHECK(build_parking_payload("A", slots, 4, 4711, 128456, tight, sizeof(tight)) == -1);

        char tiny[8];
        CHECK(build_parking_payload("A", slots, 4, 4711, 128456, tiny, sizeof(tiny)) == -1);
    }
    case_end();

    case_begin("counter wrap values pass through verbatim");
    {
        const parking_payload_slot_t slot = {1, PARKING_OCCUPIED, UINT32_MAX};
        char small[160];
        const int n = build_parking_payload("A", &slot, 1, UINT32_MAX, UINT32_MAX, small, sizeof(small));
        CHECK(n > 0);
        CHECK(strstr(small, "\"ts_ms\":4294967295") != NULL);
        CHECK(strstr(small, "\"seq\":4294967295") != NULL);
        CHECK(strstr(small, "\"changed_ms\":4294967295") != NULL);
    }
    case_end();

    case_begin("bad arguments are rejected");
    {
        parking_payload_slot_t slot = {1, PARKING_FREE, 0};
        CHECK(build_parking_payload(NULL, &slot, 1, 1, 1, buf, sizeof(buf)) == -1);
        CHECK(build_parking_payload("A", &slot, 1, 1, 1, NULL, sizeof(buf)) == -1);
        CHECK(build_parking_payload("A", &slot, 1, 1, 1, buf, 0) == -1);
        CHECK(build_parking_payload("A", NULL, 1, 1, 1, buf, sizeof(buf)) == -1);
        CHECK(build_parking_payload("A", NULL, 0, 1, 1, buf, sizeof(buf)) > 0);
    }
    case_end();

    return test_summary("payload");
}
