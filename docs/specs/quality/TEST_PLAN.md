# Test Plan

Status: **Active** (host unit tests + Wokwi simulation executed; backend and
end-to-end layers pending)

This document describes the layered testing strategy for the Smart Parking
System.

## Host Unit Tests

Pure parking-domain logic (state machine, thresholds, debouncing, recovery,
statistics) is tested on the host — no ESP32, no IDF toolchain, no QEMU
(Phase 20 host-test plan; per-phase implementation plans live in `docs/impl`,
untracked). CMake builds five `unit/*` executables whose cases
stream Redis-style `[ok]: name (N ms)` lines:

```sh
make -C firmware/esp32 test        # FULL gate: host units + all Wokwi scenarios
make -C firmware/esp32 test-host   # units only, sub-second inner loop
ctest                              # same binaries via CTest, e.g. for CI
                                   # (build tree: firmware/esp32/tests/build)
```

- **Scope**: `main/parking/parking.c` + real production config
  (`parking_config.c`) compiled against stubs of `esp_err.h`, `esp_log.h`,
  `driver/gpio.h`, and a scriptable ultrasonic stand-in. The debug injection
  hook compiles out (`CONFIG_PARKING_DEBUG_INJECT` undefined).
- **Units**:
  - `unit/classification` — threshold boundaries 30/35, band semantics
    (29/34/36), hysteresis holds.
  - `unit/debounce` — confirmation sequences `[25,25]`, `[25,40,25,25]`,
    `[40,33,36]`, `[28,28,45,45]`, flip-flop noise.
  - `unit/errors` — NaN/-5/0/450 rejection via the scan path, failing sensor
    does not abort the scan, mark_error idempotency, invariant
    `total = occupied + available + error`.
  - `unit/recovery` — pre-error restore (silent vs event), immediate
    non-debounced recovery, resumed debounce after recovery.
  - `unit/statistics` — mixed-state counters, NULL conventions, state names.
- **Sync hazard**: `stubs/ultrasonic_stub.c` mirrors the driver's plausibility
  policy; `ULTRASONIC_MIN_TOLERANCE_CM` lives in `ultrasonic.h` so both share
  one constant. Keep the stub in sync when touching the driver.
- **Responsibility split**: host tests cover decision logic; Wokwi scenarios
  cover timing, GPIO wiring, and the serial pipeline end-to-end.

### Shared case sources, on-target self-test

The classification/debounce/recovery/statistics case bodies live in
`main/test/cases_*.c` (portable Redis-style framework in
`main/test/parking_selftest.h`) and are compiled into BOTH the host binaries
and the production firmware. In the firmware they sit behind
`CONFIG_PARKING_DEBUG_INJECT`; the debug console's `selftest` serial command
runs them under FreeRTOS and prints `=== SELFTEST PASSED ===`, verified
headlessly by `tests/wokwi-scenario-selftest.yaml`:

```sh
make scenario SCENARIO=tests/wokwi-scenario-selftest.yaml   # on-target run
```

Release builds never call the units, so linker garbage-collection strips
them. Only `unit/errors` is host-only — it scripts sensor failures through
the ultrasonic stub, which real hardware cannot do.

## Firmware

- **Unit tests**: driver and service logic.
- **State-machine tests**: occupancy transitions per `../decisions/ADR-0007-parking-state-model.md`
  (UNKNOWN → FREE/OCCUPIED, hysteresis, ERROR recovery).
- **Driver tests**: HC-SR04 driver behavior (init, invalid args, timeout,
  invalid response).
- **Sensor validation**: handling of valid/invalid/ambiguous readings (`FR-012`).
- **Error handling**: timeout, sensor disconnect, task failure.

## Simulation

- **Wokwi boot**: firmware boots in Wokwi (see `../decisions/ADR-0003-wokwi.md`);
  scenario `firmware/esp32/tests/wokwi-scenario.yaml`.
- **Sensor behavior**: `wokwi-hc-sr04` distance changes reflected in firmware.
  Diagram wiring is audited against `slot_configs[]` (GPIOs 5/18, 19/21, 22/23).
- **Headless state matrix**: `tests/wokwi-scenario-injection.yaml` drives the
  full occupancy/error matrix through the `PARKING_DEBUG_INJECT` serial hook
  (`setdist <slot> <cm>`; build-time gated in `Kconfig.projbuild`, compiled out
  for release). Injected values traverse validate → EMA filter → debounce
  exactly like real measurements. Covers: all free, per-slot occupation,
  freeing via override, multiple occupied, hysteresis band hold, persistent
  invalid-measurement ERROR, and recovery with event. Record:
  `verifications/sensor-integration.md`.
- **State transitions**: FREE ⇄ OCCUPIED events observed on the serial log
  (`Slot N became OCCUPIED/FREE` + lot summary invariant).
- **Connectivity**: device connection behavior (once protocol decided).

## Backend

- **Unit tests**: service logic.
- **API tests**: device ingestion and availability endpoints.
- **Persistence tests**: current state and history storage.

## Integration

```text
ESP32
 ↓
Backend
 ↓
Database
 ↓
Dashboard
```

Verify the full data path from device to dashboard.

## End-to-End

Test realistic parking scenarios:

- Vehicle arrives and slot becomes occupied.
- Vehicle leaves and slot becomes free.

Include failure scenarios:

- Sensor disconnected
- Sensor returns invalid data
- ESP32 reboot
- Wi-Fi failure
- Backend unavailable
- Duplicate message
- Delayed message
- Device reconnect

## Verification Records

Each executed verification shall be recorded in `verifications/` using
`../templates/VERIFICATION.md`.

## Related Documents

- `../product/REQUIREMENTS.md`
- `../planning/PLAN.md`
- `../templates/VERIFICATION.md`
