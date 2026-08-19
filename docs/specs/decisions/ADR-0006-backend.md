# ADR-0006: Backend

Status: Proposed

## Context

The backend ingests device state, maintains slot state, and serves availability
to the dashboard. A Rust workspace exists at `server/parking-server/` with a
starter `main.rs` that prints "Hello, world!" and no dependencies.

## Problem

What backend technology, boundaries, and API style should the system use?

## Decision

**Partially decided / Pending Decision.**

The repository establishes a **Rust** backend scaffold:

- `server/parking-server/Cargo.toml` — package `server`, edition 2024.
- `server/parking-server/src/main.rs` — starter binary, no dependencies.

Not yet decided (each is Pending Decision):

- Web framework
- Service boundaries
- API style
- Persistence technology
- Authentication
- Device ingestion protocol (see `ADR-0005`)
- Parking state management implementation

## Decision Details

- **Language**: Rust (established by repository).
- **Package**: `server` at `server/parking-server/`.
- **Current state**: starter binary only; no API, persistence, or services.

## Alternatives Considered

- Other backend languages/frameworks were not adopted; Rust is already
  scaffolded in the repository.

## Consequences

### Positive

- Rust provides performance and safety for a networked service.
- Scaffold already present reduces setup friction.

### Negative

- Framework, persistence, and API style are undefined, so the backend milestone
  (M4) cannot begin implementation yet.
- No concurrency model (sync vs. async) is selected yet.

## Validation

- Backend architecture shall be recorded in `../architecture/backend-architecture.md`.
- Implementation verified by backend and integration tests (`../quality/TEST_PLAN.md`).

## Related Documents

- `../architecture/backend-architecture.md`
- `../architecture/database.md`
- `../architecture/tech-stack.md`
- `../planning/PLAN.md` (M4 — Backend)