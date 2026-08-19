# Hardware Architecture

Status: **Current** (simulation only; no physical sensor yet)

This document describes the hardware architecture of the device layer. In the
current repository, hardware exists only in Wokwi simulation: an ESP32 DevKit C
V4 with a serial monitor. No sensor is connected.

## Conceptual Diagram

```text
Parking Slot 1
      │
    Sensor
      │
      ▼
    ESP32
      │
      ├── Sensor 2      (future)
      ├── Sensor 3      (future)
      └── Sensor N      (future)
```

## Current Hardware (Simulation)

- **Board**: ESP32 DevKit C V4 (`board-esp32-devkit-c-v4` in `diagram.json`).
- **Connections**: `TX`/`RX` to a serial monitor only.
- No sensors, LEDs, or displays connected.

## Planned Hardware

| Component | Status | Notes |
| --------- | ------ | ----- |
| ESP32 board | Current | DevKit C V4 |
| Sensor | Pending Decision | See `ADR-0004` |
| GPIO | Pending Decision | Assignments only after confirmed by code/config |
| Power | Pending Decision | Not addressed yet |
| LEDs | Pending Decision | Optional status indication |
| Displays | Pending Decision | Not required for MVP |

## Physical Topology

The target maps sensors to parking slots via the device identity model
(`../decisions/ADR-0008-device-identity.md`). A device may host one or more
sensors, each associated with a slot.

## GPIO Assignments

Actual GPIO assignments are **not** documented because they have not been
determined by code or configuration. They will be recorded here once confirmed.

## Related Documents

- `../decisions/ADR-0003-wokwi.md`
- `../decisions/ADR-0004-sensor-selection.md`
- `firmware-architecture.md`
- `../product/GLOSSARY.yaml`