# Audit: Backend Reference Template Adoption

Date: 2026-08-25
Scope: introduction of `server/my-api` (rust-starter, MIT) as a local-only
reference baseline for `server/parking-server`, and the spec alignment around
it (ADR-0006, tech-stack, PLAN M4).

## Findings

1. **Repository hygiene — PASS.** `my-api/` is listed in `.git/info/exclude`
   alongside the pre-existing `PROMPT.md` and `docs/impl/` exclusions; the
   template directory carries its own `.git` history but is unreachable from
   this repo's object store. `git status` stays clean with it present.
2. **License compatibility — PASS.** Template is MIT; only concepts and
   patterns are imported into parking-server, no code copied verbatim yet.
   If whole files are ever vendored, add the MIT notice to
   `server/parking-server` (action recorded for P1+).
3. **Secrets hygiene — PASS.** `parking-server/.env` is git-ignored and holds
   only broker URI/credentials placeholders; nothing secret is tracked.
4. **Build isolation — PASS.** The template workspace is never built by this
   repo's tooling or CI; parking-server remains a single crate.
5. **Spec drift risk — ADDRESSED.** ADR-0006 previously described the backend
   as a "starter binary", contradicting shipped Phase 23 code; now records
   axum/tokio/rumqttc as decided-by-implementation, persistence explicitly
   Candidate-pending.

## Resulting Actions

- Phased import plan documented (impl PHASE-24, P1..P7) with gates required
  between steps.
- Persistence decision (SeaORM+Postgres) consciously left Pending until a
  Postgres instance exists and P5 begins.
