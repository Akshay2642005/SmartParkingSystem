# Test Plan

Status: **Planned** (testing strategy; most tests are not yet implemented)

This document describes the layered testing strategy for the Smart Parking
System.

## Firmware

- **Unit tests**: driver and service logic.
- **State-machine tests**: occupancy transitions per `../decisions/ADR-0007-parking-state-model.md`.
- **Driver tests**: sensor driver behavior.
- **Sensor validation**: handling of valid/invalid/ambiguous readings (`FR-012`).
- **Error handling**: timeout, sensor disconnect, task failure.

## Simulation

- **Wokwi boot**: firmware boots in Wokwi (see `../decisions/ADR-0003-wokwi.md`).
- **Sensor behavior**: sensor state changes reflected in firmware.
- **State transitions**: FREE ⇄ OCCUPIED transitions observed.
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