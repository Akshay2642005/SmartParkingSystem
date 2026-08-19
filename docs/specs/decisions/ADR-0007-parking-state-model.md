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
UNKNOWN
   │
   │ first valid reading
   ▼
FREE ─────────────┐
   │              │
   │ distance      │ distance ≤
   │ > free_thr    │ occupied_thr
   ▼              │
OCCUPIED ─────────┘
   │
   │ invalid reading
   ▼
ERROR (recovers to FREE/OCCUPIED on next valid reading)
```

States:

- **UNKNOWN**: startup state until a valid sensor reading is available.
- **FREE**: slot is available.
- **OCCUPIED**: slot is in use.
- **ERROR**: invalid or out-of-range reading, or sensor failure.

## Transition Semantics (firmware implementation)

- On startup, a slot begins in `UNKNOWN` (`parking_slot_init`).
- A reading at or below `occupied_threshold_cm` transitions to `OCCUPIED`.
- From `OCCUPIED`, a reading above `free_threshold_cm` transitions to `FREE`.
  The gap between `occupied_threshold_cm` (30 cm) and `free_threshold_cm`
  (40 cm) provides **hysteresis** and prevents state flapping on noise.
- A reading outside the valid range (below 2 cm or above 400 cm) sets `ERROR`.
- `ERROR` recovers to `FREE` or `OCCUPIED` on the next valid reading.
- A sensor measurement failure sets `ERROR` via `parking_slot_mark_error`.

## Behavior for Edge Conditions

| Condition | Behavior |
| --------- | -------- |
| Invalid sensor reading | `ERROR` state; no crash (`FR-012`) |
| Out-of-range reading (< 2 cm or > 400 cm) | `ERROR` state |
| Sensor timeout / measurement failure | `ERROR` via `parking_slot_mark_error` |
| Device offline | Backend retains last known state (backend not yet implemented) |
| Startup | `UNKNOWN` until a valid reading is available |
| Threshold noise / ambiguous readings | Hysteresis prevents flapping |

## Consequences

### Positive

- Clear, implemented, testable state model (`FR-002`, `FR-011`).
- Explicit startup and failure handling.

### Negative

- State semantics must be kept consistent when the backend is implemented.

## Validation

- State-machine logic is exercised in Wokwi (see
  `../quality/verifications/sensor-integration.md`).
- Backend must maintain the latest state per slot once implemented (`FR-031`).

## Related Documents

- `../product/REQUIREMENTS.md`
- `../architecture/firmware-architecture.md`
- `../architecture/database.md`
- `../planning/PLAN.md` (M2, M4)