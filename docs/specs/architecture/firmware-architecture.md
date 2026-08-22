# Firmware Architecture

Status: **Current** (M2 — Sensor Layer; local domain being hardened)

This document describes the ESP-IDF firmware architecture. The sensor driver,
parking state machine, lot model, and scan API are implemented; connectivity and
telemetry are not yet.

## Layer Structure

```text
Application (main.c) — orchestration only
    │
    ▼
Parking Domain (parking/parking.c)
    │  parking_lot_scan()
    ├── slot 1 ──► sensor + state machine + events
    ├── slot 2 ──► sensor + state machine + events
    └── slot 3 ──► sensor + state machine + events
    │
    ▼
parking_lot_update_counts() → statistics
    │
    ▼
Configuration (parking/parking_config.c)
    │
    ▼
Drivers
    └── Sensor (sensors/ultrasonic.c)
    │
    ▼
ESP-IDF / FreeRTOS
```

## Current State

### app_main

- `main/main.c` initializes the slots and runs a single periodic loop.
- It contains no parking implementation details; it only calls
  `parking_lot_scan()` each cycle (`FR-010`).

### Parking Lot Scan

- `parking_lot_scan()` in `parking/parking.c` owns the full scan cycle per
  slot: measure the ultrasonic sensor, validate the reading (invalid →
  `ERROR`, scan continues), smooth it through a per-slot EMA filter
  (`PARKING_FILTER_ALPHA`, seeded by the first valid reading), feed the stable
  distance to the state machine, process state-change events, and finally
  refresh lot statistics.

### Tasks & Scheduling

- A single task runs `app_main`'s loop with a 1 s delay between cycles.
- A dedicated parking task and configurable scan interval are planned
  (Phases 13–14).

### Parking State Machine

- Implemented in `parking/parking.c` (FREE/OCCUPIED/ERROR) with hysteresis
  (occupied 30 cm / free 35 cm).
- `ERROR` recovers deterministically on the first valid measurement; events
  fire only for real occupancy changes (recovery to the pre-error state is
  silent).
- See `../decisions/ADR-0007-parking-state-model.md`.

### Events

- `parking_slot_update()` returns a self-describing `parking_event_t` struct:

  | Field | Meaning |
  | ----- | ------- |
  | `type` | `PARKING_EVENT_NONE` / `PARKING_EVENT_SLOT_OCCUPIED` / `PARKING_EVENT_SLOT_FREED` |
  | `slot_id` | Which slot generated the event (from slot config). |
  | `distance_cm` | Measurement that caused the change. |

- A consumer can determine what happened, which slot, and why — without
  touching the slot array. `ERROR` transitions do not produce events; recovery
  emits only real occupancy changes.

  Event flow — the state machine never logs; `handle_parking_event()` is the
  single seam where consumers attach (logger today; event queue / networking
  later):

  ```text
  parking_slot_update() -> parking_event_t -> handle_parking_event()
  ```

- Events drive INFO logging; detailed measurements are logged at DEBUG.

### Lot Statistics

- Four counters on `parking_lot_t`, recomputed after every scan by
  `parking_lot_update_counts()`: total (`slot_count`), occupied, available,
  error.
- Invariant: `total = occupied + available + error`. ERROR slots are not
  bookable and are excluded from `available_count`.
- Query API: `parking_lot_get_total/_occupied/_available/_error` (all
  const-correct), plus slot read accessors `parking_slot_get_state` and
  `parking_slot_get_distance_cm` (latest EMA-filtered distance) as the
  supported read path for telemetry/tests.

### Slot Configuration

- Slot count, GPIO, and thresholds live in `parking/parking_config.c`
  (declared in `parking/parking_config.h`).

### Error Handling

- Measurement validation gates every reading before `parking_slot_update`;
  invalid readings (non-numeric, zero/negative, outside 2–400 cm) mark the slot
  `ERROR` and never become FREE or OCCUPIED.
- Measurement failures call `parking_slot_mark_error` (`FR-012`); the scan
  continues with the other slots.
- The ultrasonic driver validates arguments, GPIO wiring, and measurement
  range (2–400 cm), returning `ESP_ERR_TIMEOUT` / `ESP_ERR_INVALID_RESPONSE` /
  `ESP_ERR_INVALID_ARG` rather than crashing.
- Sensor init failure is treated as a fatal configuration error
  (`ESP_ERROR_CHECK`).

### Logging

- Tags: `parking` (state/events), `ultrasonic` (sensor lifecycle).
- Levels: INFO for events/system status, DEBUG for per-slot measurements,
  ERROR for failures.

### Configuration

- Per-slot: `id`, `trig_gpio`, `echo_gpio`, `occupied_threshold_cm`,
  `free_threshold_cm`.

### Networking

- Not yet implemented. Wi-Fi and device-to-backend communication are pending
  (`../decisions/ADR-0005-device-communication.md`).

### Telemetry

- Not yet implemented.

### Watchdog / Recovery

- Not yet implemented. Recovery from connectivity loss is planned (`FR-021`).

## Build

- CMake-based ESP-IDF build; component `main` requires `esp_driver_gpio`,
  `esp_timer`, and `log`.
- Project builds with `-Wall -Werror`.

## Related Documents

- `../decisions/ADR-0002-esp-idf.md`
- `../decisions/ADR-0007-parking-state-model.md`
- `hardware-architecture.md`
- `../quality/TEST_PLAN.md`
- `../planning/PLAN.md` (M1, M2, M3)
- `PROMPT.md` (Local Embedded Logic Improvements)