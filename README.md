# Smart Parking System

A real-time IoT parking management system that detects parking-slot occupancy
using ESP32-connected sensors, reports parking state to a backend platform,
persists parking information, and exposes current parking availability to users.

> Status: **active** — current phase **M0 (Specification)**.
> The project is in its scaffolding phase. See [docs/specs/](docs/specs/) for the
> full specification.

## Repository Layout

```text
firmware/esp32/      ESP-IDF firmware (smart_parking) + Wokwi simulation
server/parking-server/  Rust backend
client/dashboard/    Next.js dashboard
docs/specs/          Specification system (source of truth)
```

## Current State

| Area | Status |
| ---- | ------ |
| Firmware | ESP-IDF scaffold; boots and prints system info |
| Simulation | Wokwi (ESP32 + serial monitor) |
| Sensor | Pending Decision |
| Backend | Rust scaffold; no API yet |
| Dashboard | Next.js starter template |
| Specification | M0 baseline |

## Specifications

Requirements, architecture, decisions (ADRs), planning, quality, and security
are documented under [docs/specs/](docs/specs/README.md).

- Product & requirements: [docs/specs/product/REQUIREMENTS.md](docs/specs/product/REQUIREMENTS.md)
- Architecture: [docs/specs/architecture/system-architecture.md](docs/specs/architecture/system-architecture.md)
- Decisions: [docs/specs/decisions/](docs/specs/decisions/)
- Plan: [docs/specs/planning/PLAN.md](docs/specs/planning/PLAN.md)

## Getting Started

Firmware development uses ESP-IDF and Wokwi (see
[firmware/esp32/README.md](firmware/esp32/README.md)). Backend and dashboard
details will be added as those milestones begin.