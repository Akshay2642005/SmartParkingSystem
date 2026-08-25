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

Tags: `parking` (state/events), `ultrasonic` (sensor lifecycle) — nothing else.

Level convention:

| Level | Use in this project |
| ----- | ------------------- |
| `ESP_LOGE` | sensor timeout, invalid measurement (per-scan occurrences) |
| `ESP_LOGW` | unusual-but-recovered conditions (slot recovered from `ERROR`) |
| `ESP_LOGI` | occupancy events, lot summary, boot/init status |
| `ESP_LOGD` | per-slot raw/stable measurements |

Rule of thumb: *events at INFO, measurements at DEBUG*. Recovery from `ERROR`
logs WARN once; a silent band-restore to the pre-error state logs nothing
(ADR-0007). Example output:

```text
I parking: Slot 2 became OCCUPIED
I parking: Slot 2 became FREE
W parking: Slot 1 recovered from ERROR -> FREE
D parking: Slot 2 | Raw: 27.42 cm | Stable: 28.10 cm | State: OCCUPIED
```

No runtime log-level tuning (`esp_log_level_set`) or custom formatting —
defaults are fine until networking changes the output budget.

### Embedded Safety Review

Static-memory / embedded safety audit before networking (2026-08-22). Evidence
recorded per row; a row without evidence counts as failed.

| Requirement | Verdict | Evidence |
| ----------- | ------- | -------- |
| No unnecessary heap allocation | ✔ | `grep malloc\|calloc\|realloc\|strdup main/` → zero hits; slot array is `static parking_slot_t slots[PARKING_SLOT_COUNT]` in `main.c`; task stack via `xTaskCreate` is IDF-internal, accepted |
| No memory leaks | ✔ | All state is struct-resident (`parking_slot_t`, `parking_lot_t`); nothing dynamically owned, nothing to free |
| No unbounded loops | ✔ | Both ECHO waits in `ultrasonic_measure_cm` bounded by `esp_timer_get_time()` deadlines; task loop period-bounded by `vTaskDelayUntil`; no other loops |
| Sensor loops have timeouts | ✔ | Both echo waits use `ULTRASONIC_TIMEOUT_US` (30 ms) |
| No invalid pointer derefs | ✔ | NULL guards on all 10 public entry points across `parking.c` and `ultrasonic.c` (checked line-by-line) |
| No buffer overflows | ✔ | Only indexed array is `slots[i]`, `i < lot->slot_count`; tables sized `[PARKING_SLOT_COUNT]` at compile time |
| No recursion | ✔ | All 16 functions in `main/` are flat calls; call graph reviewed, no self/mutual recursion |
| Bounded execution time | ✔ | Worst-case scan = 3 × (trigger ~15 µs + rise wait 30 ms + fall wait 30 ms) ≈ **180 ms** ≪ 1000 ms scan period; typical scan ≈ tens of µs per slot |
| Const-correct configuration | ✔ | `const parking_slot_config_t slot_configs[]`; all four query getters take `const parking_lot_t*` / `const parking_slot_t*` (Phase 9 audit) |
| Appropriate integer types | ✔ | `float` distances · `uint8_t` ids/counts · `size_t` cardinalities · `int64_t` µs timing · `TickType_t` scheduling · `esp_err_t` error codes |
| No unnecessary large copies | ✔ | One ~20 B config copy at init (`slot->config = *config`); events passed by value ≤ 12 B; everything else by pointer |

Stack headroom (measured in Wokwi, 2026-08-22): parking task high water mark
**3316 words free of 4096 (81 %)** after the deepest scan + logging path —
well above the 25 % requirement; `PARKING_TASK_STACK_SIZE` unchanged. The
probe logs once at DEBUG (`PARKING_STACK_PROBE_AFTER_SCANS`).

Concurrency: single-writer contract documented on `parking_lot_t` — networking
phases must add locking before cross-task readers.

### Configuration

Three-file map — every knob has exactly one definition site:

| File | Owns |
| ---- | ---- |
| `parking/parking_config.h/.c` | hardware + per-slot wiring: slot count, `id`, `trig_gpio`, `echo_gpio`, `occupied_threshold_cm` (30 cm), `free_threshold_cm` (35 cm). Thresholds stay here because they depend on sensor mounting height. |
| `parking/parking_settings.h` | algorithm/system defaults (header-only): scan interval, EMA alpha, confirmation counts, task stack/priority/name. |
| `sensors/ultrasonic.h` | driver-owned measurement limits: 2–400 cm range (+ near-floor tolerance), 30 ms timeout. |

No Kconfig/menuconfig integration and no per-slot settings overrides — both
deliberately deferred until a real need appears.

### Networking

Implemented (Phase 22); MQTT publishing is behind `CONFIG_PARKING_MQTT_ENABLE`
(default n; release/debug-console builds compile it out). Wi-Fi STA support
is always linked, with SSID/password from `PARKING_WIFI_*` Kconfig.

- `net/wifi_sta.c` — blocking `wifi_sta_connect()`: init, default STA netif,
  register handlers for `WIFI_EVENT` (`STA_START` -> explicit
  `esp_wifi_connect()`; IDF does not join implicitly) and
  `IP_EVENT_STA_GOT_IP`, configure SSID/password from Kconfig, start, wait on
  a binary semaphore with timeout. Retry callers re-run the whole sequence;
  the handler path is idempotent (`esp_wifi_disconnect/stop` before
  reconfigure).
- `mqtt/mqtt_publisher.c` — dedicated task; owns a depth-8 transition queue.
  esp-mqtt client from the `espressif/mqtt` managed component. On
  `MQTT_EVENT_CONNECTED`: retained `online` status + full snapshot; LWT is
  registered up front as retained `offline`. Transitions update the shadow
  and republish the snapshot immediately (coalesced by the queue);
  periodic refresh every 30 s ± 3 s keeps state fresh even without events.
  QoS 1, no send-level retries (snapshot republish covers loss).
- Verified end-to-end in Wokwi via a local `wokwigw` bridge to a public
  broker: all scenario transitions observed as seq-incrementing snapshots,
  teardown produced the retained `offline` LWT.

### Telemetry

- Device side implemented (Phase 22): section snapshots + node status on
  `parking/{site}/{section}/...` per `communication.md`.
- Backend consumption not yet implemented (M4).

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