
#pragma once

#include <stddef.h>
#include <stdint.h>

#include "parking.h"

/**
 * Protocol v1 state-snapshot encoder (ADR-0005).
 *
 * Pure function module: no IDF headers, no allocation, no clock access —
 * compiled into both the firmware and the host unit suite so the wire format
 * is pinned by tests (docs/specs/architecture/communication.md § State
 * Updates).
 *
 * A 4-slot section renders in roughly 230 bytes; publishers should pass a
 * buffer of at least 320 bytes.
 */

/** One slot's report as consumed by the builder. */
typedef struct {
    uint8_t number; /**< Slot number within the section (1-based). */
    parking_state_t state;
    uint32_t changed_ms; /**< Monotonic ms of the last observed transition. */
} parking_payload_slot_t;

typedef struct {
    char* cur;
    size_t rem;
    bool ok;
} payload_buf_t;

/**
 * Render the state snapshot into @p buf as protocol v1 JSON.
 *
 * Wire vocabulary is lowercase ("free"/"occupied"/"error") — deliberately
 * translated from the uppercase serial vocabulary via an exhaustive switch
 * (a new parking_state_t value breaks the build here until mapped).
 *
 * @param section    Section label used both in the body and in slot ids
 *                   ("A" -> "A-1"). Copied verbatim; no escaping is performed.
 * @param slots      Complete slot list; always rendered in full, in order.
 * @param slot_count Number of entries in @p slots.
 * @param seq        Per-node publish counter (passes through verbatim).
 * @param ts_ms      Node monotonic uptime in ms at publish time.
 * @param buf        Output buffer; NUL-terminated on success. On failure the
 *                   content is undefined beyond being NUL-safe.
 * @param len        Buffer size including space for the NUL.
 * @return Bytes written (excluding NUL), or -1 on bad arguments or truncation.
 */
int build_parking_payload(const char* section,
    const parking_payload_slot_t* slots,
    size_t slot_count,
    uint32_t seq,
    uint32_t ts_ms,
    char* buf,
    size_t len);
