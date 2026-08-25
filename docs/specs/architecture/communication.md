# Device Communication

Status: **Implemented (device + backend ingest)** — firmware publishes and
the Rust backend subscribes/validates as of Phases 22-23; dashboard
WebSocket fan-out lands with M5 (`ADR-0009`).

This document defines the device-to-backend communication contract: ESP32
section nodes publish occupancy over MQTT to a site-local broker; the Rust
backend subscribes to the same broker. The dashboard path is covered by
`ADR-0009` (WebSocket).

## Topology

```
[Node A: slots A-1..A-4]──┐
[Node B]──────────────────┼── MQTT ──> [Mosquitto broker (site LAN)] <──subscribe── [Rust backend]
[Node C]──────────────────┘   retain+LWT                                          │ WebSocket
                                                                            [Next.js dashboard]
```

One board serves one section (4 sensors ⇒ 4 slots), so a node maps 1:1 to a
section (`A`, `B`, …). The broker acts as the coordination point for v1; a
dedicated gateway service (authoritative state table, WAN-outage buffering) is
a deliberate non-goal until offline resilience becomes a requirement. Topics
and payloads are designed so such a gateway can be inserted without firmware
changes.

## Transport

- **Device → broker**: MQTT (3.1.1-compatible), port 1883 (dev) / 8883+TLS (later).
- **Broker**: Mosquitto on the site LAN (Raspberry Pi or development machine).
- **Broker → backend**: the backend runs its own MQTT client subscription
  (`rumqttc`); devices never talk to backend services directly.
- **Backend → dashboard**: WebSocket per `ADR-0009`.

## Topic Layout

```
parking/{site}/{section}/state     retained snapshot, published QoS 1
parking/{site}/{section}/status    retained online|offline (LWT target)
```

- `{site}` defaults to `main` in v1; the level exists for multi-site growth.
- `{section}` is the node's section letter/id (e.g. `A`).
- Both topics are **retained** so late joiners (backend restarts) immediately
  observe current truth without waiting for traffic.

## Connection Lifecycle

1. Wi-Fi associates; SNTP is best-effort (wall-clock time is NOT assumed on
   nodes — all timestamps are monotonic uptime milliseconds).
2. Node connects with a **Last Will and Testament**:
   `parking/{site}/{section}/status` → `"offline"` (retained).
3. On connect success the node publishes `status` → `"online"` (retained),
   then immediately republishes its full state snapshot.
4. On any disconnect: reconnect with exponential backoff, 1 s doubling to a
   60 s cap, with ±20 % jitter; after reconnection always perform step 3.

## State Updates

Published on every slot-state change and additionally refreshed periodically:

```json
{
  "v": 1,
  "ts_ms": 128456,
  "seq": 4711,
  "section": "A",
  "slots": [
    { "id": "A-1", "state": "occupied", "changed_ms": 128450 },
    { "id": "A-2", "state": "free",     "changed_ms": 90000 }
  ]
}
```

Field rules:

| Field        | Meaning                                                        |
| ------------ | -------------------------------------------------------------- |
| `v`          | Payload schema version (integer, starts at 1)                  |
| `ts_ms`      | Node uptime (ms) at publish time                               |
| `seq`        | Per-node monotonically increasing counter (wraps at 2^32−1)    |
| `section`    | Section identifier, must match the topic segment               |
| `slots[].id` | Full slot label including section prefix (`A-1`)               |
| `state`      | Exactly one of `free`, `occupied`, `error`                     |
| `changed_ms` | Uptime (ms) of the last observed transition for that slot      |

- States use lowercase protocol tokens (`free|occupied|error`) — the same three
  values as the firmware's `parking_state_t`, translated from the uppercase
  serial vocabulary by an exhaustive mapping pinned by unit/payload tests.
- Snapshots are complete (all four slots every time), never deltas — consumers
  can apply them blindly.
- Publish cadence: on change (debounced by the firmware's existing debounce
  stage) plus a periodic refresh every **30 s ±10 % jitter** proving liveness
  and healing any lost update.

## Telemetry / Heartbeat

The periodic 30 s snapshot doubles as the heartbeat; no separate heartbeat
message exists in v1. Broker-side keepalive interval: 30 s.

## Message Ordering

- MQTT orders messages per topic per connection at QoS 1; snapshots on a
  stable connection arrive in publish order.
- After reconnects or broker loss, ordering guarantees reset — recovered via
  retained state plus `seq`: a consumer seeing `seq` ≤ the last applied value
  for that section discards the message as stale.
- **Session reset**: a device reboot restarts `seq` at 1, so consumers MUST
  clear their stale baseline when the node's retained `status` → `offline`
  arrives (that is the liveness channel's second job). A retained `online`
  alone does not reset anything. Slot count per section is learned from the
  first accepted snapshot and enforced thereafter (deployment property, not
  a protocol constant).

## Idempotency

- Every payload is a full snapshot; applying it twice is harmless (`NFR-004`).
- `seq` guards against out-of-order application across reconnect windows.
- Backend persistence is last-write-wins keyed by `(site, section)`.

## Timeout Behavior

- In-flight publishes rely on MQTT QoS 1 acknowledgement; the client library
  retries automatically while connected.
- If the socket dies mid-publish the reconnect cycle (above) republishes the
  snapshot — no send-level timeout policy is needed in v1.

## Authentication

- v1 (dev): broker username/password, unique credentials per node, provisioned
  via Kconfig. Topic ACLs restrict each node to its own `parking/#` subtree.
- TLS (8883) and per-device certificates are deferred to hardening; scope:
  `../security/SECURITY_PLAN.md`.

## Versioning

- The `v` field carries the payload schema version; topics are stable across
  versions.
- Consumers MUST reject unknown `v` values loudly rather than guess.

## Dashboard Contract (summary — details in `../architecture/backend-architecture.md`)

- Browser opens one WebSocket to the backend and receives
  `{type:"snapshot", ...}` followed by `{type:"update", ...}` frames mirroring
  the device payload plus `server_ts`.
- Node liveness surfaces as `{type:"node_status", section:"A", status:"offline"}`.
- The command direction (`{type:"cmd", ...}` e.g. reserve) is reserved in the
  schema; v1 backends reply with a typed error for any command frame.

---

## Related Documents

- `../decisions/ADR-0005-device-communication.md`
- `../decisions/ADR-0009-real-time-updates.md`
- `../decisions/ADR-0008-device-identity.md`
- `backend-architecture.md`
- `../security/SECURITY_PLAN.md`
- `../product/REQUIREMENTS.md` (FR-020, FR-021)
