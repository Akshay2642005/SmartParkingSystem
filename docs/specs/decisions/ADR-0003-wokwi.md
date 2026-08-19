# ADR-0003: Wokwi

Status: Accepted

## Context

The firmware repository contains Wokwi configuration:

- `firmware/esp32/diagram.json` — an ESP32 DevKit C V4 board connected to a
  serial monitor.
- `firmware/esp32/wokwi.toml` — points at `build/flasher_args.json` and
  `build/smart_parking.elf`.

## Problem

How should firmware be developed and verified without requiring physical
hardware for every iteration?

## Decision

Use **Wokwi** as the hardware simulation environment for firmware development.

## Decision Details

- **Environment**: Wokwi (browser-based ESP32 simulator).
- **Current scope**: An ESP32 DevKit C V4 with a serial monitor. No sensors are
  connected yet.
- **Firmware artifact**: `build/smart_parking.elf`.

## Why Simulation Is Useful

- Hardware-independent development — no physical board required.
- Reproducible, shareable simulations.
- Sensor behavior can be simulated before hardware is selected.
- Potential for CI/test automation in the future.

## Physical Hardware vs. Wokwi Simulation

| Aspect | Wokwi simulation | Physical hardware |
| ------ | ---------------- | ----------------- |
| Availability | Instant, in-browser | Requires physical board |
| Sensor fidelity | Simulated | Real |
| Network behavior | Limited | Real Wi-Fi |
| Security relevance | Low | High |
| Reproducibility | High | Moderate |

## Limitations Compared with Physical Hardware

- Simulated sensors do not fully reproduce real-world signal characteristics.
- Wi-Fi and networking behavior are not representative of real hardware.
- Production security mechanisms are not applicable in simulation.

## Alternatives Considered

- **QEMU** (present in the devcontainer name and ESP-IDF support): less
  convenient for interactive sensor simulation than Wokwi; not used for this
  project.

## Consequences

### Positive

- Fast, reproducible firmware development.
- Enables sensor simulation before physical hardware.
- Lowers cost and barrier to contribution.

### Negative

- Cannot validate real-world sensor/network behavior.
- Simulation must not be treated as proof of production readiness.

## Validation

- `wokwi.toml` and `diagram.json` exist and reference a valid build.
- Wokwi boots the firmware (record verification in `../quality/verifications/`).

## Related Documents

- `../architecture/firmware-architecture.md`
- `../architecture/hardware-architecture.md`
- `../architecture/deployment.md`
- `../quality/TEST_PLAN.md`