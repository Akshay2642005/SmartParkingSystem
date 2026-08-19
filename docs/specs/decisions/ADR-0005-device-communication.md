# ADR-0005: Device Communication

Status: Proposed

## Context

The device layer must report parking state to the backend. No communication code
exists in the firmware, and the backend has no ingestion API. The protocol has
not been chosen.

## Problem

Which protocol should the ESP32 use to communicate with the backend?

## Decision

**Pending Decision.** No protocol has been selected. The candidate space and
evaluation criteria are recorded here.

## Candidates

- **MQTT**: lightweight pub/sub, well suited to IoT and low bandwidth.
- **HTTP/REST**: simple, universally supported, request/response.
- **WebSocket**: persistent bidirectional connection, real-time friendly.
- **CoAP**: UDP-based constrained-node protocol.

## Evaluation Criteria

| Criterion | Consideration |
| --------- | ------------- |
| Reliability | Message delivery and reconnection behavior |
| Bandwidth | Suitability for constrained devices |
| Device complexity | Firmware implementation effort |
| Backend complexity | Server implementation effort |
| Real-time behavior | Latency of state changes reaching clients |
| Authentication | Mechanism for authenticating devices |
| Observability | Ease of monitoring and debugging |

## Consequences

### Positive

- (TBD once selected.)

### Negative

- Device connectivity (M3) and backend ingestion cannot be finalized until the
  protocol is chosen.

## Validation

- The chosen protocol shall be reflected consistently in
  `../architecture/communication.md`,
  `../architecture/system-architecture.md`, and `../planning/PLAN.md`.
- Implementation shall be verified by an integration test
  (`../quality/verifications/device-backend.md`).

## Related Documents

- `../architecture/communication.md`
- `../architecture/backend-architecture.md`
- `../planning/PLAN.md` (M3 — Device Platform, M4 — Backend)