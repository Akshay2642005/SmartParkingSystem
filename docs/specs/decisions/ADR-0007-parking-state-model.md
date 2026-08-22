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
FREE ◄──────────────┐
   │                │
   │ distance       │ distance ≤
   │ ≥ free_thr     │ occupied_thr
   ▼                │
OCCUPIED ───────────┘
   │
   │ measurement failure / invalid reading
   ▼
ERROR ── first valid measurement ──► FREE / OCCUPIED / restore pre-error state
```

States:

- **FREE**: slot is available.
- **OCCUPIED**: slot is in use.
- **ERROR**: sensor measurement failure or invalid reading; occupancy is unknown.

## Transition Semantics (current firmware implementation)

- On startup, a slot begins in `PARKING_FREE` (`parking_slot_init`), with
  `state_before_error = FREE`.
- `FREE` → `OCCUPIED` when `distance_cm <= occupied_threshold_cm` (30 cm).
- `OCCUPIED` → `FREE` when `distance_cm >= free_threshold_cm` (35 cm).
- The gap between `occupied_threshold_cm` (30 cm) and `free_threshold_cm`
  (35 cm) provides **hysteresis** and prevents state flapping on noise.
- A measurement failure or invalid reading sets `ERROR` via
  `parking_slot_mark_error`; the slot is not considered FREE or OCCUPIED while
  in `ERROR`.
- `parking_slot_mark_error` is **idempotent**: the pre-error state is captured
  only on the first failure, so repeated failures cannot overwrite the recovery
  target.

### ERROR Recovery (deterministic)

On the first valid measurement after `ERROR`, `parking_slot_update()` decides:

| Recovery reading | New state | Event |
| ---------------- | --------- | ----- |
| ≤ 30 cm (`occupied_threshold`) | OCCUPIED | only if pre-error state was FREE |
| ≥ 35 cm (`free_threshold`) | FREE | only if pre-error state was OCCUPIED |
| 30–35 cm (hysteresis band) | restore pre-error state (`state_before_error`) | never |

Recovering to the same state the slot had before the failure is not an
occupancy change and emits no event; recovering to the other state means the
occupancy changed during the outage and emits the corresponding event.
Debouncing of recovery readings is out of scope (planned: occupancy debouncing).

## Events

State changes emit events (`PARKING_EVENT_SLOT_OCCUPIED`, `PARKING_EVENT_SLOT_FREED`)
returned by `parking_slot_update()`. Each event is a self-describing struct
carrying the event type, the generating slot's id, and the measurement that
caused it — consumers never need to reach back into slot state. `FREE → ERROR`
transitions do not emit an occupancy event. Recovery from `ERROR` emits an
event only for a real occupancy change (see table above).

## Behavior for Edge Conditions

| Condition | Behavior |
| --------- | -------- |
| Measurement timeout / failure | `ERROR` via `parking_slot_mark_error` (`FR-012`) |
| Device offline | Backend retains last known state (backend not yet implemented) |
| Startup | `FREE` until the first measurement completes |
| Threshold noise / ambiguous readings | Hysteresis prevents flapping |
| Sensor range validation (< 2 cm / > 400 cm) | Implemented (`parking_measurement_is_valid`); invalid readings mark the slot `ERROR` |

## Consequences

### Positive

- Clear, implemented, testable state model (`FR-002`, `FR-011`).
- Explicit failure state that never fabricates occupancy.
- Deterministic `ERROR` recovery with no spurious events.
- ERROR slots are counted separately in lot statistics and never appear as
  available.

### Negative

- A slot in the hysteresis band right after recovery silently restores its
  pre-error state; a genuinely changed occupancy inside the band is detected
  only once the reading leaves the band.

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