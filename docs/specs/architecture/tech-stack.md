# Technology Stack

Status: **Current** (inventory of what is established vs. pending)

This document inventories technologies. Each entry states its purpose, status,
version (only when known), and source of truth. Technologies are not invented
where the repository has not established them.

---

## Hardware

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| ESP32 DevKit C V4 | Development board | Current | — | `firmware/esp32/diagram.json` |
| HC-SR04 (ultrasonic) | Parking occupancy detection | Current | — | `firmware/esp32/main/sensors/ultrasonic.c`, `ADR-0004` |

## Firmware

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| ESP-IDF | Firmware framework | Current | 6.0.2 | `firmware/esp32/CMakeLists.txt`, `ADR-0002` |
| FreeRTOS | RTOS (provided by ESP-IDF) | Current | — | `main/hello_world_main.c` |
| C | Firmware language | Current | — | `main/hello_world_main.c` |

## Simulation

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Wokwi | Hardware simulation | Current | — | `firmware/esp32/wokwi.toml`, `diagram.json`, `ADR-0003` |
| pytest-embedded | Firmware tests | Current | — | `firmware/esp32/pytest_smart_parking.py` |

## Build System

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| CMake | Firmware build | Current | 3.22+ | `firmware/esp32/CMakeLists.txt` |
| Cargo | Backend build | Current | edition 2024 | `server/parking-server/Cargo.toml` |
| Bun | JS package manager | Current | 1.3.14 | `client/dashboard/package.json` |

## Development Tooling

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Devcontainer (espressif/idf) | Firmware environment | Current | — | `firmware/esp32/.devcontainer/` |
| clangd | Editor language server | Current | — | `firmware/esp32/.clangd` |
| Biome | Lint/format (dashboard) | Current | 2.4.2 | `client/dashboard/package.json` |
| Tailwind CSS | Dashboard styling | Current | 4 | `client/dashboard/package.json` |

## Backend

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Rust | Backend language | Current | edition 2024 | `server/parking-server/Cargo.toml` |
| tokio | Async runtime | Current | 1.x | `server/parking-server/Cargo.toml` |
| axum | HTTP framework | Current | 0.8 | `server/parking-server/Cargo.toml` |
| rumqttc | MQTT subscriber | Current | 0.25 | `server/parking-server/Cargo.toml` |
| Reference template | Production-pattern baseline (config/middleware/telemetry/persistence) | Local-only reference | MIT | `server/my-api` (git-excluded; import plan in impl PHASE-24) |

## Database

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| SeaORM + PostgreSQL | Data storage | Candidate (Pending Decision) | SeaORM 1.x | `ADR-0006`, reference template `server/my-api` |

## Frontend

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Next.js | Dashboard framework | Current | 16.3.1 | `client/dashboard/package.json` |
| React | UI library | Current | 19.2.8 | `client/dashboard/package.json` |
| TypeScript | Dashboard language | Current | 5 | `client/dashboard/package.json` |

## Infrastructure

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Cloud provider | Deployment | Pending Decision | — | `architecture/deployment.md` |

## Observability

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| tracing | Structured logging facade | Current | 0.1 | `server/parking-server/Cargo.toml` |
| tracing-subscriber | EnvFilter, formatters (compact/pretty/json), ChronoUtc timestamps | Current | 0.3 | `crates/telemetry/` |
| tracing-error | Span-context error layer | Current | 0.2 | `crates/telemetry/` |

## Testing

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| Wokwi | Firmware simulation testing | Current | — | `ADR-0003` |
| pytest | ESP-IDF example test | Current | — | `firmware/esp32/pytest_hello_world.py` |
| Backend/frontend test tools | Unit/API tests | Pending Decision | — | `quality/TEST_PLAN.md` |

## CI/CD

| Technology | Purpose | Status | Version | Source of truth |
| ---------- | ------- | ------ | ------- | --------------- |
| CI pipeline | Build/test automation | Pending Decision | — | `planning/PLAN.md` (M7) |

---

## Related Documents

- `system-architecture.md`
- `../decisions/ADR-0002-esp-idf.md`
- `../decisions/ADR-0003-wokwi.md`
- `../decisions/ADR-0006-backend.md`