# Verification: M2 Completion-Criteria Audit (PROMPT.md)

## Objective

Confirm every item on the PROMPT.md local-subsystem completion checklist
(line 950 onward) is implemented, tested, and documented before the project
is considered past the "local embedded domain" milestone. This closes the
M2 gate recorded in STATUS.yaml / SESSION.yaml.

## Preconditions

- PROMPT.md checklist: reliable measurement, validation, timeout handling,
  filtering, hysteresis, occupancy debounce, deterministic transitions,
  error/recovery handling, lot statistics, event representation, clean
  configuration, dedicated parking task, configurable scan interval,
  consistent logging, static-memory safety, Wokwi multi-slot simulation,
  tests for parking state logic.
- All phase impl records present under `docs/impl/firmware/`.

## Test Environment

- ESP-IDF v6.0.2, host CMake/CTest suite, Wokwi CLI v0.26.1
- Audit date: 2026-08-25 (post Phase 22)

## Procedure

For each checklist item, locate the implementing code, the test that pins
it, and the spec/impl document that records it.

## Actual Result

| # | Criterion | Implementation evidence | Test evidence |
|---|-----------|------------------------|---------------|
| 1 | Reliable ultrasonic measurement | `sensors/ultrasonic.c` pulse timing via `esp_timer`; near-floor tolerance shared via `ultrasonic_distance_is_plausible` (`ULTRASONIC_MIN_TOLERANCE_CM` 0.5) | unit/classification; Wokwi sensor-integration PASS |
| 2 | Validation | Range check `ULTRASONIC_MIN/MAX_DISTANCE_CM` (2–400 cm); invalid -> ERROR, never silent FREE/OCCUPIED (Phase 4) | unit/errors |
| 3 | Timeout handling | Echo timeout -> `ESP_ERR_TIMEOUT` invalid measurement path (`ultrasonic.h`); `wifi_sta_connect` 20 s IP timeout | unit/errors; PHASE-22 E2E |
| 4 | Filtering | Per-slot EMA, alpha `PARKING_FILTER_ALPHA` 0.3, seeded on first valid reading; invalid readings bypass (Phase 11) | unit/classification |
| 5 | Hysteresis | `occupied_threshold_cm` / `free_threshold_cm` per slot (30 / 35 cm band in `parking_config.c`) | unit/debounce |
| 6 | Occupancy debounce | `PARKING_OCCUPIED_CONFIRMATION_COUNT` / `FREE_CONFIRMATION_COUNT` = 2 consecutive; band readings reset counters (Phase 12) | unit/debounce |
| 7 | Deterministic state transitions | Explicit state machine with `state_before_error` restore and pending-state confirmation (Phase 5) | unit/recovery |
| 8 | Error/recovery handling | Policy in firmware-architecture § Error Handling Policy; recovery WARN + immediate FREE restore (Phases 15–16) | unit/errors, unit/recovery |
| 9 | Parking-lot statistics | Lot aggregate with `error_count`; total = occupied + available + errors invariant (Phase 8) | unit/statistics |
| 10 | Event representation | `parking_event_t` identifies slot + causing measurement (Phase 6); transitions now also fan out to observers (Phase 22 seam) | unit/errors observer case |
| 11 | Clean configuration | `parking_config.h/.c` hardware vs `parking_settings.h` algorithm knobs, single definition site (Phase 17) | header audit |
| 12 | Dedicated parking task | Named `parking_task`, stack/priority constants, app_main returns after setup (Phase 14) | Wokwi boot log |
| 13 | Configurable scan interval | `PARKING_SCAN_INTERVAL_MS` 1000 + `vTaskDelayUntil` absolute scheduling (Phase 13) | scenario timing |
| 14 | Consistent logging | Level table in firmware-architecture § Logging; silent band-restores stay silent (Phase 16) | doc review |
| 15 | Static-memory safety | Phase 18 audit table; all slot state statically allocated; 81 % stack headroom measured in Wokwi | measured evidence |
| 16 | Wokwi multi-slot simulation | 3× HC-SR04 diagram matching `slot_configs[]`; headless injection matrix scenario (Phase 19, compressed Phase 22) | verifications/sensor-integration.md PASS |
| 17 | Tests for parking state logic | Host CTest suite: classification, debounce, errors, recovery, statistics, payload — 34 cases green; same sources run on-target via serial `selftest` | make test (34/34 [ok]) |

Networking (Wi-Fi + MQTT, Phases 21–22) was started after items 1–17 were in
place; PROMPT.md's ordering intent ("local subsystem first") was honored even
though the close-out record postdates the networking phases.

## Evidence

- `docs/impl/firmware/PHASE-{03..22}*.md` implementation records
- `docs/specs/architecture/firmware-architecture.md` (policy sections)
- `docs/specs/quality/verifications/sensor-integration.md` (PASS)
- Host suite output: 34/34 `[ok]` (2026-08-25)

## Result

**PASS** — all 17 completion criteria are implemented and evidenced.

## Notes

The two remaining PROMPT.md phases beyond this checklist (Wi-Fi → MQTT →
Backend pipeline) are underway: device-side publishing landed in Phase 22;
backend consumption is planned as ADR-0006 / M4.
