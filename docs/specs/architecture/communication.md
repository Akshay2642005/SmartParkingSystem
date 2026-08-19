# Device Communication

Status: **Pending Decision** (protocol not chosen — see `ADR-0005`)

This document defines the device-to-backend communication contract. Because no
protocol has been selected and no communication code exists, most sections are
marked `Pending Decision` and describe the intended contract.

## Transport

**Pending Decision.** Candidate protocols: MQTT, HTTP/REST, WebSocket, CoAP
(see `ADR-0005`).

## Connection Lifecycle

- Device connects to the backend after Wi-Fi is established.
- Lifecycle specifics (reconnect, backoff): Pending Decision.

## Device Registration

- How a device introduces itself and associates with a slot.
- Identity scheme: Pending Decision (`ADR-0008`).

## Telemetry

- Device-sent operational state and measurements.
- Format and cadence: Pending Decision.

## State Updates

- Device-sent occupancy state changes (`FR-020`).
- Payload schema: Pending Decision.

## Heartbeat

- Periodic liveness signal to indicate the device is alive.
- Cadence: Pending Decision.

## Reconnect Behavior

- The device shall recover from temporary connectivity failures (`FR-021`).
- Retry/backoff policy: Pending Decision.

## Timeout Behavior

- Handling of unacknowledged sends and connection timeouts: Pending Decision.

## Retry Behavior

- Number of retries and retry intervals: Pending Decision.

## Message Ordering

- How out-of-order messages are handled: Pending Decision.
- Backend treats updates idempotently (`NFR-004`).

## Idempotency

- Duplicate state updates must not corrupt slot state.
- Mechanism: Pending Decision.

## Authentication

- How devices authenticate to the backend.
- Scope: `../security/SECURITY_PLAN.md`.

## Versioning

- How the protocol/payload schema is versioned: Pending Decision.

---

## Related Documents

- `../decisions/ADR-0005-device-communication.md`
- `../decisions/ADR-0008-device-identity.md`
- `backend-architecture.md`
- `../security/SECURITY_PLAN.md`
- `../product/REQUIREMENTS.md` (FR-020, FR-021)