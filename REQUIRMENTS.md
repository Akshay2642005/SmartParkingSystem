# Requirements

The authoritative requirements for the Smart Parking System live in the
specification system:

**→ [docs/specs/product/REQUIREMENTS.md](docs/specs/product/REQUIREMENTS.md)**

## Summary

### Functional Requirements

- **FR-001…FR-003** — Parking: identify slots, determine occupancy, maintain
  latest state.
- **FR-010…FR-012** — Firmware: sample sensors, convert to parking states,
  handle invalid readings.
- **FR-020…FR-021** — Connectivity: communicate state to backend, recover from
  failures.
- **FR-030…FR-031** — Backend: accept device updates, maintain latest state.
- **FR-040…FR-041** — Dashboard: display availability, identify occupied/available.

### Non-Functional Requirements

- **NFR-001** Reliability
- **NFR-002** Availability
- **NFR-003** Performance
- **NFR-004** Recoverability
- **NFR-005** Observability
- **NFR-006** Security
- **NFR-007** Maintainability
- **NFR-008** Testability

Numeric targets are not yet established and are marked `Pending Decision` in the
specifications.

## Status

- FR-001…FR-003 (Parking) and FR-010…FR-012 (Firmware) are **Current** at the
  device layer (ultrasonic driver + parking state machine).
- FR-020…FR-041 (Connectivity, Backend, Dashboard) remain **Planned**.
- The project is in phase **M2 (Sensor Layer)**. See
  [docs/specs/planning/STATUS.yaml](docs/specs/planning/STATUS.yaml).