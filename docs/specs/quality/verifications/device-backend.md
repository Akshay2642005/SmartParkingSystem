# Verification: Device-Backend Ingest over MQTT

## Objective

Verify the Phase 23 backend ingests protocol v1 snapshots from the broker
into its state store per ADR-0005: retained cold start, stale-seq guard,
reboot session reset, and graceful rejection of malformed input.

## Preconditions

- Local Mosquitto listening on localhost:1883
- `server/parking-server` built from this tree (rumqttc subscriber +
  axum `/internal/state`)
- Contract: `docs/specs/architecture/communication.md`

## Test Environment

- macOS, Rust 1.97.1 (edition 2024), mosquitto 2.x clients
- Date: 2026-08-25

## Procedure / Expected vs Actual

| # | Drill | Expected | Actual |
| - | ----- | -------- | ------ |
| 1 | Publish retained snapshot seq=5 + `status online`, then subscribe with the backend already up | snapshot ingested without new device traffic | `accepted parking snapshot seq=5 slot_count=3`; `/internal/state` returns the section |
| 2 | Republish same topic with seq=4 | rejected as stale, store unchanged | WARN `stale sequence number: seq=4, last_seq=5` |
| 3 | Retained `status offline` then fresh-session snapshot seq=1 | baseline cleared by LWT; seq=1 accepted (device reboot drill) | `node status update status=Some("offline")` then `accepted parking snapshot seq=1` |
| 4 | Publish `{invalid json` on a state topic | loud reject, task survives | WARN malformed JSON; `/internal/state` still serving |
| 5 | Unit-level reject table (`cargo test`) | all rows pass | 14/14 pass |

## Evidence

- Backend log excerpt captured in the session record; commands:
  `cargo run`, `mosquitto_pub -r -q 1`, `curl localhost:8080/internal/state`
- `cargo test`: `test result: ok. 14 passed`
- `cargo clippy --all-targets`: clean; `cargo fmt --check`: clean

## Result

**PASS**

## Notes

Drill 3 is the contract amendment recorded in communication.md § Message
Ordering: a consumer clears its stale baseline when the node's retained
`offline` arrives, because device-side `seq` restarts each boot.
