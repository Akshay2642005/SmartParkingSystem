# Database

Status: **Planned / Pending Decision** (no persistence technology chosen)

This document defines the conceptual data model for the Smart Parking System. It
is technology-agnostic; the concrete database is Pending Decision.

## Conceptual Entities

```text
Site
ParkingLot
ParkingSlot
Device
Sensor
OccupancyEvent
DeviceHeartbeat
```

| Entity | Description | MVP |
| ------ | ----------- | --- |
| Site | A physical location hosting parking lots | Post-MVP |
| ParkingLot | A collection of parking slots | Post-MVP |
| ParkingSlot | A single parkable location | Yes |
| Device | An ESP32 edge device | Yes |
| Sensor | A hardware component measuring a slot | Yes |
| OccupancyEvent | A record of a state transition | Yes |
| DeviceHeartbeat | Periodic liveness record | Yes |

## Relationships

```text
Site
 │
 └── ParkingLot
       │
       └── ParkingSlot
              │
              └── Device/Sensor
```

A device/sensor is associated with a parking slot via the identity model
(`../decisions/ADR-0008-device-identity.md`). Multiple sensors may be associated
with one device.

## Current State vs. Historical Events

The model distinguishes:

```text
Current State      → the latest occupancy of each slot (fast reads)
Historical Events  → OccupancyEvent records of every transition (audit/analytics)
```

- **Current state** is derived from the latest accepted update.
- **Historical events** retain every state transition over time.

## Persistence Technology

**Pending Decision.** See `../decisions/ADR-0006-backend.md`.

## Related Documents

- `../decisions/ADR-0006-backend.md`
- `../decisions/ADR-0007-parking-state-model.md`
- `../decisions/ADR-0008-device-identity.md`
- `backend-architecture.md`
- `../product/REQUIREMENTS.md` (FR-003, FR-031)