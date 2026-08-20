# ADR-0007: Parking State Model

Status: Accepted

## Context

The system must represent whether a parking slot is occupied and how that state
changes over time. The state machine is implemented in the firmware
(`firmware/esp32/main/parking/parking.c`).

## Problem

How should a parking slot's occupancy be modeled as a state machine?

## Decision

Use the following occupancy state machine, implemented in the firmware:

```text
FREE ─────────────┐
   │              │
   │ distance      │ distance ≤
   │ ≥ free_thr    │ occupied_thr
   ▼              │
OCCUPIED ─────────┘
   │
   │ measurement failure
   ▼
ERROR  (sticky until a future recovery phase re-measures the slot)
```

States:

- **FREE**: slot is available.
- **OCCUPIED**: slot is in use.
- **ERROR**: sensor measurement failure; occupancy is unknown.

## Transition Semantics (current firmware implementation)

- On startup, a slot begins in `PARKING_FREE` (`parking_slot_init`).
- `FREE` → `OCCUPIED` when `distance_cm <= occupied_threshold_cm` (30 cm).
- `OCCUPIED` → `FREE` when `distance_cm >= free_threshold_cm` (35 cm).
- The gap between `occupied_threshold_cm` (30 cm) and `free_threshold_cm`
  (35 cm) provides **hysteresis** and prevents state flapping on noise.
- A measurement failure sets `ERROR` via `parking_slot_mark_error`; the slot is
  not considered FREE or OCCUPIED while in `ERROR`.
- `ERROR` is currently **sticky**: no recovery transition exists yet.
  Deterministic `ERROR → FREE` / `ERROR → OCCUPIED` recovery is planned (Phase 5
  of the Local Embedded Logic Improvements) and must not emit spurious
  OCCUPIED/FREED events.

## Events

State changes emit events (`PARKING_EVENT_SLOT_OCCUPIED`, `PARKING_EVENT_SLOT_FREED`)
returned by `parking_slot_update()`. `ERROR` transitions do not emit an
occupancy event. Event representation is evolving in Phase 6 of the Local
Embedded Logic Improvements.

## Behavior for Edge Conditions

| Condition | Behavior |
| --------- | -------- |
| Measurement timeout / failure | `ERROR` via `parking_slot_mark_error` (`FR-012`) |
| Device offline | Backend retains last known state (backend not yet implemented) |
| Startup | `FREE` until the first measurement completes |
| Threshold noise / ambiguous readings | Hysteresis prevents flapping |
| Sensor range validation (< 2 cm / > 400 cm) | Planned (Phase 4) — not yet implemented |

## Consequences

### Positive

- Clear, implemented, testable state model (`FR-002`, `FR-011`).
- Explicit failure state that never fabricates occupancy.

### Negative

- `ERROR` slots have no automatic recovery yet; recovery requires the planned
  Phase 5 state-machine work.

## Validation

- State-machine logic is exercised in Wokwi (see
  `../quality/verifications/sensor-integration.md`).
- Backend must maintain the latest state per slot once implemented (`FR-031`).

## Related Documents

- `../product/REQUIREMENTS.md`
- `../architecture/firmware-architecture.md`
- `../architecture/database.md`
- `../planning/PLAN.md` (M2, M4)
- `PROMPT.md` (Phase 5 — Improve Parking Slot State Machine)