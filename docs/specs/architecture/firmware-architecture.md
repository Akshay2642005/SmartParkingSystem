# Firmware Architecture

Status: **Current** (describes the intended architecture; current code is a
boot/print scaffold)

This document describes the ESP-IDF firmware architecture. The current firmware
(`firmware/esp32/main/hello_world_main.c`) only initializes the system and prints
chip info. The layered architecture below is the **target** for the firmware and
is `Planned` until implemented.

## Target Layer Structure

```text
Application
    │
    ▼
Domain / Parking State
    │
    ▼
Services
    ├── Sensor Manager
    ├── Connectivity
    ├── Telemetry
    └── Configuration
    │
    ▼
Drivers
    ├── Sensor
    ├── GPIO
    └── Other hardware
    │
    ▼
ESP-IDF / FreeRTOS
```

## Current State

- `app_main` in `main/hello_world_main.c` runs once at boot.
- Uses FreeRTOS task APIs and ESP-IDF native APIs.
- A single task loops and prints status every second.
- No sensors, state machine, connectivity, or telemetry yet.

## Planned Components

### app_main

- Entry point; initializes the system, configuration, services, and starts
  firmware tasks.

### Tasks & Scheduling

- FreeRTOS tasks for sensor sampling, connectivity, and telemetry.
- Scheduling priorities/periods: Pending Decision.

### Sensor Sampling

- Periodically sample the connected sensor (`FR-010`).
- Driver interface converts raw readings into a normalized input.

### State Transitions

- Convert sensor-derived readings into parking states (`FR-011`) per the state
  machine in `../decisions/ADR-0007-parking-state-model.md`.

### Error Handling

- Handle invalid sensor readings without crashing (`FR-012`).
- Timeouts and ambiguous readings handled per `ADR-0007`.

### Logging

- Use ESP-IDF logging for observability (`NFR-005`).
- Config: Pending Decision.

### Configuration

- Device identity, network credentials, and sensor parameters.
- Scheme: Pending Decision (`../decisions/ADR-0008-device-identity.md`).

### Networking

- Wi-Fi initialization and connection.
- Protocol: Pending Decision (`../decisions/ADR-0005-device-communication.md`).

### Telemetry

- Send device state to the backend.
- Not yet implemented.

### Watchdog / Recovery

- Recovery from connectivity loss and task hangs (`FR-021`).
- If applicable; design pending.

## Related Documents

- `../decisions/ADR-0002-esp-idf.md`
- `../decisions/ADR-0007-parking-state-model.md`
- `hardware-architecture.md`
- `../quality/TEST_PLAN.md`
- `../planning/PLAN.md` (M1, M2, M3)