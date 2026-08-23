# System Architecture

Status: **Current** (M0 — target architecture; most components are Planned)

This document describes the overall system architecture. It is the reference for
how the device, connectivity, backend, persistence, and presentation layers fit
together. Components that are not yet implemented are marked `Planned`.

See `../decisions/ADR-0001-architecture.md` for the layering decision.

---

## Context Diagram

```text
Users
  │
  ▼
Dashboard
  │
  ▼
Backend
  │
  ▼
Database

ESP32 Devices
  │
  ▼
Backend
```

## Container-Level Architecture

```text
ESP32 firmware   → sensors + state machine + connectivity  (Planned)
Backend/API      → ingestion + state + availability        (Planned)
Database         → current state + history                  (Pending Decision)
Frontend         → dashboard availability                   (Planned)
Infrastructure   → hosting/network                          (Pending Decision)
```

| Container | Status | Notes |
| --------- | ------ | ----- |
| ESP32 firmware | Partial | Boots, samples 3 ultrasonic sensors, runs parking state machine; no connectivity yet |
| Backend/API | Scaffold | Rust starter binary; no API |
| Database | Pending Decision | No persistence technology chosen |
| Frontend | Scaffold | Next.js default starter |
| Infrastructure | Pending Decision | No cloud provider chosen |

## Data Flow

```text
Sensor           (Current — HC-SR04, Wokwi + driver)
  ↓
ESP32            (Current — parking state machine)
  ↓
Device state     (Current — in-memory on device)
  ↓
Network          (Current — MQTT via site broker, ADR-0005)
  ↓
Backend          (Planned — Rust scaffold only)
  ↓
Database         (Pending Decision)
  ↓
Dashboard        (Planned — Next.js scaffold only)
```

## Failure Paths

| Failure | Effect | Handling |
| ------- | ------ | -------- |
| Sensor disconnected | No new readings | Firmware marks slot uncertain / ERROR (`ADR-0007`) |
| Sensor invalid data | Bad reading | Firmware ignores/handles without crash (`FR-012`) |
| Device offline | No state updates | Backend retains last known state |
| Network loss | Message not delivered | Device retries / recovers (`FR-021`) |
| Backend unavailable | Ingestion down | Device retries; dashboard reflects last known state |
| Duplicate / delayed messages | Redundant updates | Backend treats updates idempotently (`NFR-004`) |

## Related Documents

- `../decisions/ADR-0001-architecture.md`
- `../decisions/ADR-0005-device-communication.md`
- `../decisions/ADR-0009-real-time-updates.md`
- `firmware-architecture.md`
- `backend-architecture.md`
- `database.md`
- `../product/VISION.md`