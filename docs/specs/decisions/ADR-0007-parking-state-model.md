# ADR-0007: Parking State Model

Status: Proposed

## Context

The system must represent whether a parking slot is occupied, and how that state
changes over time. No state machine exists in the firmware or backend yet.

## Problem

How should a parking slot's occupancy be modeled as a state machine?

## Decision

Use the following occupancy state machine (proposed):

```text
UNKNOWN
   │
   ▼
FREE
   │
   │ vehicle detected
   ▼
OCCUPIED
   │
   │ vehicle leaves
   ▼
FREE
```

- **UNKNOWN**: initial/startup state; no reliable sensor reading yet.
- **FREE**: slot is available.
- **OCCUPIED**: slot is in use.
- **ERROR**: an invalid or ambiguous condition was detected.

## State Transition Semantics

- On startup, a slot begins in `UNKNOWN`.
- `FREE` ⇄ `OCCUPIED` transitions are driven by sensor-derived vehicle
  detection.
- `ERROR` represents invalid sensor readings or sensor timeout, not a valid
  occupancy.

## Behavior for Edge Conditions

| Condition | Behavior |
| --------- | -------- |
| Invalid sensor reading | Do not change valid occupancy; optionally record an ERROR/log; do not crash |
| Sensor timeout | Mark slot state as uncertain / ERROR; do not fabricate occupancy |
| Device offline | Backend retains the last known state; no new transitions are received |
| Startup | Begin in UNKNOWN until a valid reading is available |
| Ambiguous readings | Hold current state and avoid flapping between FREE/OCCUPIED |

## Consequences

### Positive

- Clear, testable state model (`FR-002`, `FR-011`).
- Handles startup and failure conditions explicitly.

### Negative

- State semantics must be implemented consistently in both firmware and backend.

## Validation

- State-machine unit tests in firmware (`../quality/TEST_PLAN.md`).
- Backend maintains the latest state per slot (`FR-003`, `FR-031`).

## Related Documents

- `../product/REQUIREMENTS.md`
- `../architecture/firmware-architecture.md`
- `../architecture/database.md`
- `../planning/PLAN.md` (M2, M4)