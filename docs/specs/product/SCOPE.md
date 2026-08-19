# Smart Parking System — Scope

Status: **Current** (M0 — Specification baseline)

This document defines what is in scope and out of scope for the Smart Parking
System. Items are marked by their current project state. The repository is in
initial scaffolding; most capability is `Planned`.

---

## In Scope

| Area | Status | Notes |
| ---- | ------ | ----- |
| Parking slot detection | Current | HC-SR04 ultrasonic per slot (`ADR-0004`) |
| ESP32 firmware | Current | ESP-IDF `smart_parking`; samples sensors, runs state machine |
| Sensor integration | Current | Driver + Wokwi simulation (3 sensors) |
| Device state management | Current | Parking state machine (`ADR-0007`) |
| Device connectivity | Planned | Protocol pending (`ADR-0005` Proposed) |
| Telemetry | Planned | Not yet implemented |
| Backend ingestion | Planned | Rust workspace scaffolded; no API yet |
| Parking state management (backend) | Planned | Not yet implemented |
| Persistence | Planned | Not yet implemented |
| Parking availability | Planned | Not yet implemented |
| Dashboard | Planned | Next.js scaffolded; default starter only |
| Wokwi simulation | Current | `diagram.json` + `wokwi.toml`; ESP32 + 3× HC-SR04 |
| System specification | Current | This specification set |

## Out of Scope

The following are **explicitly out of scope**:

- Payment processing
- Automated license-plate recognition
- Computer vision
- Physical gate / barrier manufacturing and control
- Production hardware certification

## MVP

The MVP delivers a single working path from sensor to dashboard:

- ESP32 firmware determines slot occupancy from a sensor.
- The device communicates state to the backend.
- The backend ingests, persists, and maintains slot state.
- A dashboard displays occupied vs. available slots.

## Post-MVP

- Historical occupancy and analytics
- Driver-facing live availability
- Multiple sites and parking lots
- Fleet/OTA management
- Authentication and multi-user roles

## Deferred Features

The following are intentionally deferred and not part of the MVP:

- Payment processing
- License-plate recognition / computer vision
- Physical gate control
- Advanced analytics and forecasting
- Production hardware certification

## Explicit Non-Goals

- The project does **not** implement payment or reservation in the MVP.
- The project does **not** attempt to build physical parking infrastructure.
- The project does **not** require production-grade device security in the Wokwi
  simulation environment (see `../security/DEVICE_SECURITY.md`).

---

## Related Documents

- `VISION.md`
- `REQUIREMENTS.md`
- `../planning/PLAN.md`
