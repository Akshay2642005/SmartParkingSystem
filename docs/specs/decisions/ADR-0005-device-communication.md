# ADR-0005: Device Communication

Status: Accepted (2026-08-23)

## Context

The device layer must report parking state to the backend. No communication
code exists in the firmware, and the backend has no ingestion API. The system
deploys one ESP32 board per garage section (4 ultrasonic sensors ⇒ 4 slots),
with multiple sections per site.

## Problem

Which protocol should the ESP32 use to communicate with the backend, and where
does coordination between section nodes happen?

## Decision

**MQTT**, with a site-local Mosquitto broker acting as the coordination point:

- Section nodes publish retained QoS 1 snapshots to `parking/{site}/{section}/state`.
- Node liveness uses MQTT Last Will (`.../status` → `offline`).
- The Rust backend subscribes to the broker with its own MQTT client; devices
  never address backend services directly.
- A separate coordinator/gateway service is **not** built in v1. The broker is
  the coordinator. If WAN-outage buffering or a local authoritative state table
  becomes a requirement, a gateway service can be inserted behind the broker
  without firmware changes (topics and payloads already support it).
- Backend ↔ dashboard transport is defined separately in `ADR-0009`.

Full contract: `../architecture/communication.md`.

## Candidates

- **MQTT**: lightweight pub/sub, well suited to IoT and low bandwidth.
- **HTTP/REST**: simple, universally supported, request/response.
- **WebSocket**: persistent bidirectional connection, real-time friendly.
- **CoAP**: UDP-based constrained-node protocol.

## Rationale

| Criterion          | MQTT                          | HTTP/REST            | WebSocket            | CoAP                  |
| ------------------ | ----------------------------- | -------------------- | -------------------- | --------------------- |
| Reliability        | QoS levels + retained + LWT   | App-level retries    | App-level retries    | Limited in practice   |
| Bandwidth          | Minimal (2-byte fixed header) | Per-request overhead | Low after handshake  | Minimal               |
| Device complexity  | Low (esp-mqtt is bundled)     | Lowest               | Medium               | Medium (no IDF stack) |
| Backend complexity | Low (rumqttc subscriber)      | Low                  | Medium               | High                  |
| Real-time behavior | Push on publish               | Poll only            | Push                 | Push                  |
| Late joiners       | Retained messages solve it    | N/A                  | Manual replay needed | Manual replay needed  |
| Multi-node fan-out | Native (broker)               | N/A                  | Point-to-point only  | Native                |

Decisive points: retained messages give instant truth to late joiners (backend
restarts become non-events); LWT gives node-death detection for free; and the
broker decouples N nodes from the backend without custom code. HTTP was
rejected because polling cannot detect offline nodes and multiplies firmware
reconnect logic. WebSocket device-side was rejected as redundant once MQTT
covers the same needs with less firmware code. CoAP offered no advantage that
outweighed ecosystem maturity.

## Consequences

### Positive

- Firmware gains push-based reporting with mature IDF tooling (`esp-mqtt`).
- Broker retention removes backend cold-start and replay problems.
- LWT gives free, reliable node-liveness for the dashboard.
- Adding sections/sites is purely additive (new topics), zero config changes.

### Negative

- Operational dependency: a broker process must run somewhere (site LAN /
  Raspberry Pi) — one more thing to monitor.
- QoS 1 permits duplicates; consumers must be idempotent (handled via full
  snapshots + `seq` guard).
- Plaintext 1883 in dev defers TLS/auth hardening to a later phase
  (`SECURITY_PLAN.md`).

## Validation

- The chosen protocol shall be reflected consistently in
  `../architecture/communication.md`, `../architecture/system-architecture.md`,
  and `../planning/PLAN.md`.
- Implementation shall be verified by an integration test
  (`../quality/verifications/device-backend.md`): a published snapshot must
  appear in the backend's state store and on a connected dashboard WebSocket.

## Related Documents

- `../architecture/communication.md`
- `../decisions/ADR-0009-real-time-updates.md`
- `../decisions/ADR-0008-device-identity.md`
- `../architecture/backend-architecture.md`
- `../planning/PLAN.md` (M3 — Device Platform, M4 — Backend)
