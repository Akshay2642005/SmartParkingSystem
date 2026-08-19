# Hardware Architecture

Status: **Current** (Wokwi simulation; physical hardware not yet deployed)

This document describes the hardware architecture of the device layer. The
current hardware exists in Wokwi simulation: an ESP32 DevKit C V4 with three
HC-SR04 ultrasonic sensors.

## Conceptual Diagram

```text
Parking Slot 1   Parking Slot 2   Parking Slot 3
      │                │                │
    Sensor1          Sensor2          Sensor3
      │                │                │
      └──────┬─────────┴─────────┬──────┘
             ▼
           ESP32
```

## Current Hardware (Simulation)

- **Board**: ESP32 DevKit C V4 (`board-esp32-devkit-c-v4` in `diagram.json`).
- **Sensors**: three `wokwi-hc-sr04` ultrasonic sensors.
- **Connections**: `TX`/`RX` to a serial monitor; each sensor wired to VCC
  (5 V), GND, TRIG, and ECHO.

## GPIO Assignments

Confirmed by `firmware/esp32/diagram.json` and
`firmware/esp32/main/parking/parking_config.c`:

| Slot | TRIG GPIO | ECHO GPIO | Occupied ≤ | Free > |
| ---- | --------- | --------- | ---------- | ------ |
| 1 | 5 | 18 | 30 cm | 40 cm |
| 2 | 19 | 21 | 30 cm | 40 cm |
| 3 | 22 | 23 | 30 cm | 40 cm |

## Physical Topology

Each parking slot has one HC-SR04 sensor connected to the ESP32. Slot identity
(`id`) maps a sensor to a slot; the device identity model
(`../decisions/ADR-0008-device-identity.md`) is pending.

## Power

- Sensors are powered from the 5 V rail in Wokwi.
- Physical power design: Pending Decision.

## Planned Hardware (Not Yet Present)

| Component | Status | Notes |
| --------- | ------ | ----- |
| Connectivity (Wi-Fi) | Pending Decision | See ADR-0005 |
| LEDs / status indicators | Pending Decision | Optional |
| Displays | Pending Decision | Not required for MVP |
| Production provisioning | Pending Decision | See `../security/DEVICE_SECURITY.md` |

## Related Documents

- `../decisions/ADR-0003-wokwi.md`
- `../decisions/ADR-0004-sensor-selection.md`
- `firmware-architecture.md`
- `../product/GLOSSARY.yaml`