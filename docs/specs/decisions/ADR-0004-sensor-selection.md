# ADR-0004: Sensor Selection

Status: Proposed

## Context

The system must determine whether a parking slot is occupied. A sensor measures
the physical state of a slot and feeds the ESP32 firmware. Currently no sensor
is connected in the repository (`diagram.json` contains only an ESP32 + serial
monitor), and no sensor driver exists in the firmware.

## Problem

Which sensor type should be used to detect parking-slot occupancy?

## Decision

**Pending Decision.** No sensor has been selected. A decision must be made
before the sensor layer milestone (M2) is implemented. This ADR records the
candidate space and evaluation criteria.

## Candidates

- **Ultrasonic** (e.g. HC-SR04): measures distance via sound; commonly used for
  presence.
- **Infrared (IR)**: detects reflected IR to infer presence.
- **Magnetic**: detects the presence of metal (vehicle body) near the slot.
- **Time-of-Flight (ToF)**: measures distance via light.

## Evaluation Criteria

| Criterion | Consideration |
| --------- | ------------- |
| Cost | Per-slot cost and total system cost |
| Accuracy | Reliability of distinguishing occupied vs. free |
| Environmental sensitivity | Effect of weather, temperature, dirt, lighting |
| Wiring | GPIO/interface complexity with the ESP32 |
| ESP32 compatibility | Ease of integration with ESP-IDF |
| Simulation support | Whether Wokwi can simulate the sensor |
| Maintainability | Calibration and long-term reliability |

## Consequences

### Positive

- (TBD once selected.)

### Negative

- Development of the sensor layer (M2) is blocked until selection is made.
- A wrong choice may require rework of the firmware sensor driver.

## Validation

- Once selected, a sensor driver shall be implemented and verified in Wokwi
  (`../quality/verifications/sensor-integration.md`).

## Related Documents

- `../architecture/hardware-architecture.md`
- `../architecture/firmware-architecture.md`
- `../planning/PLAN.md` (M2 — Sensor Layer)