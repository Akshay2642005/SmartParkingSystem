# Test Plan

Status: **Planned** (testing strategy; most tests are not yet implemented)

This document describes the layered testing strategy for the Smart Parking
System.

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