# ADR-0004: Sensor Selection

Status: Accepted

## Context

The system must determine whether a parking slot is occupied. A sensor measures
the physical state of a slot and feeds the ESP32 firmware.

## Problem

Which sensor type should be used to detect parking-slot occupancy?

## Decision

Use an **HC-SR04 ultrasonic sensor** (one per parking slot).

This is confirmed by the repository:

- `firmware/esp32/main/sensors/ultrasonic.c` implements an HC-SR04 driver
  (trigger/echo timing, constant sound-speed conversion).
- `firmware/esp32/diagram.json` connects three `wokwi-hc-sr04` parts to the
  ESP32.
- `firmware/esp32/main/parking/parking_config.c` maps one sensor per slot.

A slot is considered **occupied** when the measured distance is at or below
`occupied_threshold_cm` (30 cm) and **free** when at or above `free_threshold_cm`
(35 cm). Hysteresis between the two thresholds prevents state flapping on
noise. The driver converts the ECHO round-trip time to a one-way distance using
a constant sound speed (0.0343 cm/µs). Occupancy decisions consume an
EMA-smoothed distance (per-slot filter, `PARKING_FILTER_ALPHA`), so threshold
comparisons act on stable readings rather than raw jitter.

## Driver Limits and Error Semantics

The driver (`sensors/ultrasonic.c`) defines the following named constants and
returns the corresponding `esp_err_t` values instead of crashing:

| Aspect | Value |
| ------ | ----- |
| Minimum distance | 2 cm (`ULTRASONIC_MIN_DISTANCE_CM`) |
| Maximum distance | 400 cm (`ULTRASONIC_MAX_DISTANCE_CM`) |
| Measurement timeout | 30 ms (`ULTRASONIC_TIMEOUT_US`) |
| Sound speed | 0.0343 cm/µs (`SOUND_SPEED_CM_PER_US`) |
| TRIG pulse / idle | 10 µs / 2 µs |

- Invalid arguments or invalid GPIO wiring (out of range, TRIG == ECHO) return
  `ESP_ERR_INVALID_ARG` from `ultrasonic_init()`.
- GPIO configuration or init failures are propagated as errors, not fatal.
- A missing ECHO pulse (or a distance beyond 400 cm) returns `ESP_ERR_TIMEOUT`.
- An implausible reading (below 2 cm, or an inconsistent pulse) returns
  `ESP_ERR_INVALID_RESPONSE`.

These errors are handled upstream by the parking domain (`FR-012`): a failed
measurement marks the slot `ERROR` and the scan continues.

## Alternatives Considered

| Sensor | Consideration |
| ------ | ------------- |
| Ultrasonic (HC-SR04) | Chosen — low cost, good range, easily simulated in Wokwi, simple GPIO interface |
| Infrared | Lower cost but range/sensitivity to ambient light and surface reflectivity |
| Magnetic | Good for metal detection but requires placement under vehicle and is harder to simulate |
| Time-of-Flight | Accurate but more expensive and less commonly simulated in Wokwi |

### Evaluation Criteria Applied

- **Cost**: HC-SR04 is inexpensive.
- **Accuracy**: sufficient to distinguish occupied vs. free at parking distance.
- **Environmental sensitivity**: acceptable for a parking slot (mounted above/at
  the slot); documented as a limitation.
- **Wiring**: two GPIOs per sensor (TRIG, ECHO).
- **ESP32 compatibility**: native GPIO driver (`esp_driver_gpio`).
- **Simulation support**: `wokwi-hc-sr04` is supported in Wokwi.
- **Maintainability**: per-slot thresholds configured in one place.

## Consequences

### Positive

- Low-cost, well-supported sensor with Wokwi simulation parity.
- Per-slot configuration of GPIO and thresholds.

### Negative

- Ultrasonic sensing is sensitive to debris, weather, and mounting angle.
- One sensor per slot increases per-slot cost and wiring.

## Validation

- Firmware samples each sensor and derives occupancy in Wokwi
  (see `../quality/verifications/sensor-integration.md`).
- GPIO assignments match `firmware/esp32/diagram.json` and
  `firmware/esp32/main/parking/parking_config.c`.

## Related Documents

- `../architecture/hardware-architecture.md`
- `../architecture/firmware-architecture.md`
- `../decisions/ADR-0003-wokwi.md`
- `../planning/PLAN.md` (M2 — Sensor Layer)