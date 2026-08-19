# ADR-0009: Real-Time Updates

Status: Proposed

## Context

The dashboard must reflect parking availability. It is not yet clear whether
availability must update in real time or whether periodic refresh is sufficient.
The dashboard is a Next.js starter with no data fetching or update mechanism.

## Problem

How should the dashboard receive updated parking availability?

## Decision

**Pending Decision.** Whether real-time updates are required has not been
determined. If they are required, the mechanism is also undecided. The candidate
space is recorded here.

## Candidates

- **Polling**: client periodically re-requests availability. Simple; not truly
  real-time; acceptable for many use cases.
- **Server-Sent Events (SSE)**: server pushes updates over a single HTTP
  connection. Simple, one-way, real-time.
- **WebSockets**: full-duplex persistent connection; real-time both directions;
  more complex.
- **Pub/Sub**: (e.g. via the communication layer) decoupled delivery; requires
  infrastructure.

## Evaluation Criteria

- Requirement: does the product require near-immediate updates?
- Latency: how quickly must a state change reach the dashboard?
- Complexity: client and server implementation effort.
- Consistency with the chosen device communication layer (`ADR-0005`).

## Consequences

### Positive

- (TBD once decided.)

### Negative

- Dashboard milestone (M5) data layer cannot be finalized until the mechanism is
  chosen.

## Validation

- The chosen mechanism shall be reflected in `../architecture/backend-architecture.md`
  and `../architecture/system-architecture.md`.
- Verified by integration/end-to-end tests (`../quality/TEST_PLAN.md`).

## Related Documents

- `../architecture/backend-architecture.md`
- `../architecture/communication.md`
- `../planning/PLAN.md` (M5 — Dashboard)