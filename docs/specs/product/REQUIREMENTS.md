# Smart Parking System — Requirements

Status: **Current** (M2 — Sensor Layer)

This document specifies functional and non-functional requirements using stable
identifiers. Requirements describe **intended** behavior. Whether a requirement
is currently implemented is determined by the repository source code and tracked
in `../planning/STATUS.yaml`.

Status of each requirement: `Planned` unless noted otherwise.

---

## Functional Requirements

### Parking

- **FR-001** — The system shall identify individual parking slots.
  Status: Current (per-slot `id` in `parking_config.c`)
- **FR-002** — The system shall determine the occupancy state of each parking
  slot.
  Status: Current (ultrasonic driver + parking state machine)
- **FR-003** — The system shall maintain the latest known state of each slot.
  Status: Partial — firmware maintains in-memory state; backend persistence is
  Planned

### Firmware

- **FR-010** — The ESP32 shall periodically sample connected sensors.
  Status: Current (1 s sampling loop)
- **FR-011** — The firmware shall convert sensor readings into parking states.
  Status: Current (`parking_slot_update`)
- **FR-012** — The firmware shall handle invalid sensor readings without
  crashing.
  Status: Partial — measurement failures set `ERROR` and the scan continues;
  measurement range validation is Planned (Phase 4)

### Connectivity

- **FR-020** — The device shall communicate its state to the backend.
  Status: Planned (protocol pending, see `ADR-0005`)
- **FR-021** — The device shall recover from temporary connectivity failures.
  Status: Planned

### Backend

- **FR-030** — The backend shall accept device state updates.
  Status: Planned
- **FR-031** — The backend shall maintain the latest state of each device and
  slot.
  Status: Planned

### Dashboard

- **FR-040** — The dashboard shall display parking availability.
  Status: Planned
- **FR-041** — The dashboard shall identify occupied and available slots.
  Status: Planned

---

## Non-Functional Requirements

Numeric targets are **Pending Decision** and are only added here once
established by the project.

- **NFR-001 — Reliability**: The system shall handle invalid sensor data and
  device failures without crashing.
- **NFR-002 — Availability**: The system shall reflect the latest known state of
  slots and recover from transient backend/network unavailability.
- **NFR-003 — Performance**: The system shall maintain slot state within targets
  agreed for the platform. Specific targets: Pending Decision.
- **NFR-004 — Recoverability**: Devices and the backend shall recover from
  restart, connectivity loss, and out-of-order/duplicate messages.
- **NFR-005 — Observability**: The system shall provide logging to diagnose
  device and backend behavior.
- **NFR-006 — Security**: The system shall protect device identity, credentials,
  and data in transit and at rest (see `../security/`).
- **NFR-007 — Maintainability**: The system shall be separated into firmware,
  backend, and dashboard with clear boundaries and documentation.
- **NFR-008 — Testability**: The system shall be testable at firmware,
  simulation, backend, integration, and end-to-end levels (see
  `../quality/TEST_PLAN.md`).

---

## Related Documents

- `SCOPE.md`
- `VISION.md`
- `../planning/STATUS.yaml`
