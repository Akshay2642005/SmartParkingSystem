# ADR-0009: Real-Time Updates

Status: Accepted (2026-08-23)

## Context

The dashboard must reflect parking availability. The dashboard is a Next.js
starter with no data fetching or update mechanism. Device-side transport is
now decided (`ADR-0005`): MQTT snapshots reach the Rust backend via a broker
subscription. The remaining hop is backend → browser.

## Problem

How should the dashboard receive updated parking availability?

## Decision

**WebSocket** between the Rust backend and the Next.js dashboard, using a
snapshot-then-deltas contract:

1. On connect, the backend sends `{type:"snapshot"}` with full current state.
2. Every accepted device update is fanned out as `{type:"update"}` frames.
3. Node liveness (MQTT LWT) surfaces as `{type:"node_status"}` frames.
4. Client→server frames use a reserved `{type:"cmd", ...}` envelope; v1
   replies with a typed error for any command, but the schema keeps the door
   open for reserve/gate-control features without a protocol break.

Polling and SSE were considered and rejected.

## Candidates

- **Polling**: client periodically re-requests availability.
- **Server-Sent Events (SSE)**: server pushes over one HTTP connection.
- **WebSockets**: full-duplex persistent connection.
- **Pub/Sub**: decoupled delivery requiring extra client infrastructure.

## Rationale

- The command direction (reservations, gate control) is a plausible product
  step; SSE would force a second transport the moment it ships. WebSocket
  covers both directions with one mechanism.
- Polling fails the product's core promise: a garage availability board must
  update within moments, not on a refresh timer, and polling N sections per
  client scales poorly.
- A raw pub/sub client in the browser (MQTT-over-WSS) was rejected: it would
  expose broker credentials to browsers and bypass backend auth/persistence.

## Consequences

### Positive

- One uniform real-time story end-to-end (device push → backend fan-out).
- Command channel exists from day one, even if unused in v1.
- Snapshot-on-connect makes dashboard refresh/reconnect trivial (no state
  reconciliation logic in React).

### Negative

- Backend owns connection management (heartbeats, dead-socket cleanup).
- Slightly more code than SSE for the read-only case (~a message-type enum).

## Validation

- Integration test (`../quality/verifications/device-backend.md`): publishing a
  device snapshot to the broker must produce an `update` frame on an open
  dashboard WebSocket within 1 s; a fresh connection must receive a correct
  `snapshot` frame.

## Related Documents

- `../architecture/communication.md`
- `../decisions/ADR-0005-device-communication.md`
- `../product/REQUIREMENTS.md`
