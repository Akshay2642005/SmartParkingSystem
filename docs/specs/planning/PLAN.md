# Implementation Plan

Status: **Current** (M0 — Specification)

This is the milestone-based implementation plan. Milestones may be adjusted as
the project evolves. Currently only M0 is active.

```text
M0 — Specification        (current)
M1 — ESP32 Foundation
M2 — Sensor Layer
M3 — Device Platform
M4 — Backend
M5 — Dashboard
M6 — End-to-End Integration
M7 — Production Hardening
```

---

## M0 — Specification

- **Objective**: Establish system requirements and architecture.
- **Prerequisites**: None.
- **Deliverables**: This specification set (docs/specs/).
- **Acceptance criteria**: Requirements, architecture, ADRs, and planning
  documents exist and are internally consistent.
- **Risks**: Decisions left pending may delay later milestones.

## M1 — ESP32 Foundation

- **Objective**: Establish firmware and simulation foundation.
- **Prerequisites**: M0.
- **Deliverables**: Firmware structure; clean `app_main`; Wokwi boot
  verification.
- **Acceptance criteria**: Firmware builds and boots in Wokwi; verification
  recorded.
- **Status**: Completed (builds clean with `-Wall -Werror`).

## M2 — Sensor Layer

- **Objective**: Integrate a sensor and derive occupancy.
- **Prerequisites**: M1; sensor selection (`ADR-0004`).
- **Deliverables**: Sensor driver; sampling task; parking state machine
  (`ADR-0007`).
- **Acceptance criteria**: Firmware detects occupancy states in Wokwi;
  invalid-readings handling verified.
- **Status**: In progress — driver, sampling, and state machine implemented;
  Wokwi verification pending.
- **Risks**: Sensor selection and simulation fidelity.

## M3 — Device Platform

- **Objective**: Add device connectivity, telemetry, and identity.
- **Prerequisites**: M2; protocol selection (`ADR-0005`); identity scheme
  (`ADR-0008`).
- **Deliverables**: Connectivity layer; telemetry; config; reconnect behavior.
- **Acceptance criteria**: Device sends state to a backend and recovers from
  connectivity failure.
- **Risks**: Protocol and identity decisions.

## M4 — Backend

- **Objective**: Implement backend ingestion, state, and persistence.
- **Prerequisites**: M3; backend framework + database selection (`ADR-0006`).
- **Deliverables**: Device ingestion; parking-slot service; persistence.
- **Acceptance criteria**: Backend accepts device updates and maintains latest
  state (`FR-030`, `FR-031`).
- **Risks**: Framework/persistence choices.

## M5 — Dashboard

- **Objective**: Implement parking availability dashboard.
- **Prerequisites**: M4; real-time mechanism (`ADR-0009`).
- **Deliverables**: Availability view; occupied/available display.
- **Acceptance criteria**: Dashboard displays current slot state (`FR-040`,
  `FR-041`).
- **Risks**: Real-time update decision.

## M6 — End-to-End Integration

- **Objective**: Verify the full device → backend → database → dashboard path.
- **Prerequisites**: M3, M4, M5.
- **Deliverables**: Integration and end-to-end tests.
- **Acceptance criteria**: Realistic parking scenarios pass end-to-end.
- **Risks**: Cross-layer contract mismatches.

## M7 — Production Hardening

- **Objective**: Prepare for production deployment and security.
- **Prerequisites**: M6; cloud/infrastructure decisions.
- **Deliverables**: Deployment config; security hardening; CI/CD.
- **Acceptance criteria**: Staging/production deployment runs reliably; security
  mitigations applied.
- **Risks**: Infrastructure and provisioning effort.

---

## Related Documents

- `PHASES.yaml`
- `STATUS.yaml`
- `RELEASES.yaml`
- `../product/SCOPE.md`
- `../product/REQUIREMENTS.md`