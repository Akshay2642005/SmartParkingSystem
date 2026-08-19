# Firmware Architecture

Status: **Current** (M2 — Sensor Layer partially implemented)

This document describes the ESP-IDF firmware architecture. The sensor and
parking-state modules are implemented; connectivity and telemetry are not yet.

## Target Layer Structure

```text
Application (main.c)
    │
    ▼
Domain / Parking State (parking/parking.c)
    │
    ▼
Services
    ├── Sensor Manager      (main.c update_slots)
    ├── Connectivity        (not yet implemented)
    ├── Telemetry           (not yet implemented)
    └── Configuration       (parking/parking_config.c)
    │
    ▼
Drivers
    ├── Sensor              (sensors/ultrasonic.c)
    ├── GPIO                (via esp_driver_gpio)
    └── Other hardware      (none yet)
    │
    ▼
ESP-IDF / FreeRTOS
```

## Current State

### app_main

- `main/main.c` initializes slots and runs a single loop.
- Each loop iteration samples every sensor and updates slot state.

### Tasks & Scheduling

- A single task runs `app_main`'s loop with a 1 s delay between cycles.
- Per-slot sampling is sequential.
- Dedicated FreeRTOS tasks for connectivity/telemetry: not yet implemented.

### Parking State Machine

- Implemented in `parking/parking.c` (UNKNOWN/FREE/OCCUPIED/ERROR).
- See `../decisions/ADR-0007-parking-state-model.md`.

### Slot Configuration

- Slot count, GPIO, and thresholds live in `parking/parking_config.c`
  (declared in `parking/parking_config.h`).

### Sensor Sampling

- `main.c:update_slots` samples each sensor via `ultrasonic_measure_cm` and
  passes the distance to `parking_slot_update` (`FR-010`, `FR-011`).

### Error Handling

- Invalid/out-of-range readings set `ERROR` via `parking_slot_update`.
- Measurement failures call `parking_slot_mark_error` (`FR-012`).
- Invalid GPIO configuration is rejected by `ultrasonic_init`.

### Logging

- `ESP_LOGI` per slot update and `ESP_LOGE` on sensor errors (TAG `parking`).
- Provided by the `log` component (declared in `main/CMakeLists.txt`).

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