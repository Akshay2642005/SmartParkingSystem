# Backend Architecture

Status: **Planned** (scaffold exists; architecture intended)

This document describes the intended backend architecture. The current backend is
a Rust starter binary (`server/parking-server/src/main.rs`) with no API. See
`../decisions/ADR-0006-backend.md`.

## Intended API Split

```text
Device-facing API   → device state ingestion
User-facing API     → availability queries for the dashboard
Internal services   → state management, persistence
```

## Planned Components

### API

- Endpoints for device ingestion and availability queries.
- Style and framework: Pending Decision.

### Services

- Device ingestion service.
- Parking-slot state service.
- Authentication/authorization (MVP scope: `../security/SECURITY_PLAN.md`).

### Device Ingestion

- Accept device state updates (`FR-030`).
- Validate and attribute updates to slots (`ADR-0008`).
- Implemented as an MQTT subscriber (Phase 23) — validation rules and
  reject taxonomy are contractual: `communication.md` § Backend Ingest
  Rules.

### Parking-Slot Service

- Maintain the latest state of each slot (`FR-031`).
- Enforce the state machine in `ADR-0007`.

### Persistence

- Store current state and occupancy history.
- Technology: Pending Decision (`database.md`).

### Events

- Produce events for state changes (for real-time updates / analytics).
- Mechanism: Pending Decision.

### Real-Time Updates

- Push availability changes to the dashboard.
- Mechanism: WebSocket, snapshot-then-deltas (`ADR-0009`); the backend also
  subscribes to the device broker per `ADR-0005`.

## Current State

- Rust package `server` (edition 2024).
- `main.rs` prints "Hello, world!".
- No HTTP framework, services, or persistence.

## Related Documents

- `../decisions/ADR-0006-backend.md`
- `../decisions/ADR-0009-real-time-updates.md`
- `communication.md`
- `database.md`
- `../product/REQUIREMENTS.md` (FR-030, FR-031)