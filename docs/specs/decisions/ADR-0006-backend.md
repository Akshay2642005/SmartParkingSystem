# ADR-0006: Backend

Status: Proposed

## Context

The backend ingests device state, maintains slot state, and serves availability
to the dashboard. A Rust service exists at `server/parking-server/` and, as of
Phase 23, already implements MQTT ingest (tokio + axum + rumqttc), a validated
in-memory state store, and a debug HTTP endpoint.

A production-pattern reference template (rust-starter, MIT) is staged
locally at `server/my-api` — excluded from git by design — providing the
conventions the project imports incrementally: layered configuration,
tower-http middleware, structured telemetry, SeaORM/Postgres persistence,
OpenAPI, auth scaffolding.

## Problem

Which backend conventions are binding, which parts of the reference template
get adopted, and what remains open (persistence technology foremost)?

## Decision

**Partially decided.**

Decided by implementation (Phase 23) and recorded here as of 2026-08-25:

- **Language**: Rust, edition 2024.
- **Async runtime**: tokio.
- **Web framework**: axum 0.8 (`server/parking-server/Cargo.toml`).
- **Device ingestion**: rumqttc subscriber per `ADR-0005`.
- **Reference baseline**: `server/my-api` (local-only) supplies patterns;
  features land in parking-server stepwise per the phased import plan in
  `docs/impl/server/PHASE-24-backend-template-adoption.md`.

Still Pending Decision:

- Persistence technology — **candidate**: SeaORM + PostgreSQL (the template's
  stack), gated on P5 of the import plan; requires this ADR's acceptance
  plus a provisioned Postgres instance.
- Service boundaries beyond ingest/state/query (single binary for v1).
- Authentication (template's better-auth pattern noted for later).

## Alternatives Considered

- Other backend languages/frameworks were not adopted; Rust is already
  scaffolded in the repository.
- sqlx directly (without SeaORM): lighter, but the template standardizes on
  SeaORM migrations/entities; revisit only if the ORM fights the schema.
- Vendoring the whole template workspace: rejected — parking-server stays a
  single crate until size demands splitting; concepts over code.

## Consequences

### Positive

- Rust provides performance and safety for a networked service.
- Scaffold already present reduces setup friction.

### Negative

- Persistence is still undecided, so history/queries remain in-memory only
  and M4 cannot close until P5 lands.
- Importing template patterns piecemeal risks half-adopted conventions if
  steps are skipped — the phased plan mitigates but requires discipline.

## Validation

- Backend architecture shall be recorded in `../architecture/backend-architecture.md`.
- Implementation verified by backend and integration tests (`../quality/TEST_PLAN.md`).

## Related Documents

- `../architecture/backend-architecture.md`
- `../architecture/database.md`
- `../architecture/tech-stack.md`
- `../planning/PLAN.md` (M4 — Backend)