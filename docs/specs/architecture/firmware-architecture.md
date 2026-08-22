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
  distance to the state machine — where transitions require
  `PARKING_*_CONFIRMATION_COUNT` consecutive agreeing readings (debouncing) —
  process state-change events, and finally refresh lot statistics.

### Tasks & Scheduling

- A dedicated parking task (`parking_task`, named `PARKING_TASK_NAME`) owns
  the scan cycle: `xTaskCreate` from `app_main` with a 4 KiB stack
  (`PARKING_TASK_STACK_SIZE`) and priority 5 (`PARKING_TASK_PRIORITY`),
  above idle/log workers and below critical system tasks.
- The task is scheduled with `vTaskDelayUntil` against
  `PARKING_SCAN_INTERVAL_MS` (1 s): absolute-period scheduling keeps the scan
  rate deterministic regardless of how long each scan takes (worst case ≈
  90 ms for three slots at full timeout vs. the 1000 ms period).
- `app_main` initializes slots, creates the task, and returns — it does not
  participate in the scan loop. Task-creation failure aborts via
  `ESP_ERROR_CHECK`.

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

### Error Handling Policy

Two error classes with distinct handling — fail fast at init, isolate at
runtime:

| Error class | Examples | Handling | Rationale |
| ----------- | -------- | -------- | --------- |
| Fatal init | bad GPIO config (`ultrasonic_init` pin validation), `gpio_config` failure, task creation failure | `ESP_ERROR_CHECK` → abort/boot loop with message | system cannot fulfill its only purpose; fail fast and visibly |
| Recoverable runtime | ultrasonic timeout, implausible reading, TRIG pulse failure mid-measurement | slot → `ERROR`, log, continue scanning other slots | one flaky sensor must not restart the whole device |

Runtime failure flow:

```text
measurement timeout / invalid reading / trigger GPIO failure
      ↓
parking_slot_mark_error()   (idempotent, remembers pre-error state)
      ↓
ESP_LOGE in parking_lot_scan()
      ↓
continue scanning other slots
      ↓
deterministic recovery on first valid measurement (ADR-0007)
```

- Exactly three fatal sites exist, all init-time: slot initialization
  (`initialize_slots` wrapping `parking_slot_init`), task creation (the
  `xTaskCreate` result checked via `ESP_ERROR_CHECK`), and the scan wrapper in
  `app_main`'s task (safe because `parking_lot_scan` returns `ESP_OK` even when
  individual slots error).
- The driver never aborts: every `ultrasonic_measure_cm` failure path —
  including a TRIG GPIO that stops accepting writes at runtime — returns an
  `esp_err_t` code. No retries/backoff inside the driver; recovery is the
  state machine's job.
- Watchdog / panic handlers are not configured yet; revisit with networking.

- Measurement validation gates every reading before `parking_slot_update`;
  invalid readings (non-numeric, zero/negative, outside the plausible band)
  mark the slot `ERROR` and never become FREE or OCCUPIED. Plausibility has a
  single source of truth: `ultrasonic_distance_is_plausible()` in the sensor
  layer, shared by driver and parking layer. The band accepts a small
  tolerance below the datasheet floor (readings ≥ ~1.5 cm): at 2 cm the echo
  round trip is only ~116 µs, so microsecond quantization and edge-detection
  jitter would otherwise flip legitimate near-floor readings to errors.
- Measurement failures call `parking_slot_mark_error` (`FR-012`); the scan
  continues with the other slots.
- The ultrasonic driver validates arguments, GPIO wiring, and measurement
  plausibility (2–400 cm with the near-floor tolerance), returning
  `ESP_ERR_TIMEOUT` / `ESP_ERR_INVALID_RESPONSE` / `ESP_ERR_INVALID_STATE` /
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