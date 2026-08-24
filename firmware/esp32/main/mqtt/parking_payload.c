#include "parking_payload.h"
#include "parking.h"

#include <stdarg.h>
#include <stdio.h>

static const char* payload_state_name(parking_state_t state) {
    switch (state) {
        case PARKING_FREE:
            return "free";
        case PARKING_OCCUPIED:
            return "occupied";
        case PARKING_ERROR:
            return "error";
    }
    return "error";
}

__attribute__((format(printf, 2, 3))) static void buf_addf(payload_buf_t* b, const char* fmt, ...) {
    if (!b->ok)
        return;
    va_list args;
    va_start(args, fmt);

    const int n = vsnprintf(b->cur, b->rem, fmt, args);

    if (n < 0 || (size_t)n >= b->rem) {
        b->ok = false;
        return;
    }
    b->cur += n;
    b->rem -= (size_t)n;
}

int build_parking_payload(const char* section, const parking_payload_slot_t* slots, size_t slot_count, uint32_t seq, uint32_t ts_ms, char* buf, size_t len) {
    if (section == NULL || buf == NULL || len == 0 || (slots == NULL && slot_count > 0)) {
        return -1;
    }

    payload_buf_t b = {buf, len, true};

    buf_addf(&b,
        "{\"v\":1,\"ts_ms\":%lu,\"seq\":%lu,\"section\":\"%s\",\"slots\":[",
        (unsigned long)ts_ms,
        (unsigned long)seq,
        section);

    for (size_t i = 0; i < slot_count && b.ok; ++i) {
        buf_addf(&b,
            "%s{\"id\":\"%s-%u\",\"state\":\"%s\",\"changed_ms\":%lu}",
            (i > 0) ? "," : "",
            section,
            slots[i].number,
            payload_state_name(slots[i].state),
            (unsigned long)slots[i].changed_ms);
    }

    buf_addf(&b, "]}");

    if (!b.ok) {
        return -1;
    }

    return (int)(b.cur - buf);
}
