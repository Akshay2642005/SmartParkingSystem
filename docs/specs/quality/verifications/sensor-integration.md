# Verification: Sensor Integration in Wokwi

## Objective

Verify that the HC-SR04 driver, measurement validation, EMA filtering,
debounced occupancy state machine, error handling, and lot statistics work
together correctly in the simulated system — and that the full state matrix
from PROMPT.md (Phase 19) can be exercised headlessly via `wokwi-cli`.

## Preconditions

- Firmware built with `CONFIG_PARKING_DEBUG_INJECT=y`
  (`firmware/esp32/main/Kconfig.projbuild`), enabling the `setdist` serial
  hook (`debug_console.c`).
- Diagram `wokwi-diagram.json`: three `wokwi-hc-sr04` parts wired to
  TRIG/ECHO = 5/18, 19/21, 22/23; sensor 1 at 2 cm, sensors 2–3 at 100 cm.
- Thresholds from production config: occupied ≤ 30 cm, free ≥ 35 cm
  (hysteresis band 30–35), debounce N = 2 confirmations, scan period 1 s,
  EMA alpha = 0.3.

## Test Environment

- ESP-IDF v6.0.2, Wokwi CLI v0.26.1, wokwi-cli scenario runner
- Host: macOS (darwin), project at firmware/esp32

## Procedure

1. Wiring audit: compare `wokwi-diagram.json` part pins against
   `slot_configs[]` in `parking_config.c`.
2. Boot smoke test: `wokwi-cli --scenario tests/wokwi-scenario.yaml`.
3. Full matrix: `wokwi-cli --timeout 45000 --scenario
   tests/wokwi-scenario-injection.yaml --serial-log-file <log>`. Injected values
   enter the pipeline ahead of validation and traverse filter + debounce like
   real measurements; consecutive injections are required because single-shot
   values are absorbed by the EMA filter (stable = 25 + 75·0.7^k crosses the
   30 cm threshold on scan 8; debounce fires on scan 9).

## Expected Result

| # | Matrix row | Expected evidence |
|---|------------|-------------------|
| 1 | all free | boot summary `Occupied: 0 \| Available: 3 \| Errors: 0` |
| 2 | slot 1 occupied | real sensor @2 cm → `Slot 1 became OCCUPIED` |
| 3 | slot 2 occupied | injected 25 cm ×9 → event after filter+debounce only |
| 4 | slot 3 occupied | injected 25 cm ×9 → `Slot 3 became OCCUPIED` |
| 5 | slot 1 freed | injected 100 cm override ×3 → `Slot 1 became FREE` |
| 6 | multiple occupied | summary `Occupied: 2` while slots 1+2 occupied |
| 7 | sensor timeout/error | NaN ×2 → `E ... Invalid measurement`, `Errors: 1` persists, other slots unaffected |
| 8 | recovery from sensor error | first valid reading → immediate recovery; pre-error OCCUPIED ⇒ `became FREE` + `W ... recovered from ERROR -> FREE` |

Additionally: hysteresis band readings (33 cm) must produce no transition;
band-side readings while settled must reset confirmation counters silently.

## Actual Result

All eight rows observed as expected in a single headless run
(`Scenario completed successfully`, CLI exit 0). Key timestamps:

- `(285)` `Occupied: 0 | Available: 3` — all free at boot ✔ (row 1)
- `(1275)` `Slot 1 became OCCUPIED` ✔ (row 2)
- `(10275)` `Slot 2 became OCCUPIED` after exactly the predicted ~9 injected
  scans ✔ (row 3); `Occupied: 2` through `(11285)–(14285)` ✔ (row 6)
- Band hold: injections of 33 cm at scans `(11275)–(13275)` produced **no**
  events — filter equilibrium sits mid-band, counters reset every scan ✔
- `(14275)`/`(15275)` two consecutive `Invalid measurement: nan cm`, summary
  `Errors: 1` persisting across both scans, slot counts adjust
  (`Occupied: 2 → 1`) while slot 1 stays OCCUPIED ✔ (row 7)
- `(16275)` `Slot 2 became FREE` + `W: Slot 2 recovered from ERROR -> FREE`
  on the first valid reading — recovery not debounced, change vs pre-error
  state emits both event and WARN ✔ (row 8)
- `(25285)` `Slot 3 became OCCUPIED` ✔ (row 4)
- `(28275)` `Slot 1 became FREE` after three 100 cm overrides — first lands
  mid-band (0.3·100 + 0.7·2 ≈ 32), then two confirmed candidates ✔ (row 5)
- Lot invariant `Total = Occupied + Available + Errors` holds on every
  periodic summary line throughout.

Negative checks: stopping injections lets healthy real readings legitimately
resume control of a slot (slot 3 relaxed to FREE at `(27285)` once its
injections ceased) — documented pipeline behavior, not a defect.

## Evidence

- Scenario: `firmware/esp32/tests/wokwi-scenario-injection.yaml` (all steps matched;
  `Scenario completed successfully`)
- Serial log excerpts quoted above (timestamps in ms of simulated time);
  full log retained in session workspace `wokwi-p19-full.log`
- Wiring audit: diagram GPIOs match `slot_configs[]` for all three slots
- Earlier iterations that exposed the injection-dynamics constraints
  (EMA absorption of single shots; one-scan self-heal of isolated NaN; delay
  steps swallowing one-shot serial lines) are recorded in the scenario header
  comments.

## Result

PASS

## Notes

- The debug hook is compiled out unless `CONFIG_PARKING_DEBUG_INJECT=y`;
  release builds do not expose `setdist`.
- `wait-serial` does not rewind the serial stream: one-shot lines emitted
  during a `delay` step are lost. Transition waits therefore sit immediately
  after the triggering write (periodic status lines can tolerate delays).
- Free-tier simulation time is finite; scenarios are kept short (~31 s) and
  deterministic.
