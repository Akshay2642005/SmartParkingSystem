# ADR-0001: System Architecture

Status: Accepted

## Context

The Smart Parking System spans edge hardware, networking, and software. Without a
clear separation of concerns, device firmware, backend logic, and user-facing
views would become entangled, making the system hard to build, test, and extend.

## Problem

How should the system be structured so that each part has a clear responsibility,
can fail independently, and can be developed and tested in isolation?

## Decision

Separate the system into logical layers:

```text
Device Layer
      ↓
Connectivity Layer
      ↓
Platform/Backend Layer
      ↓
Persistence Layer
      ↓
Presentation Layer
```

- **Device Layer**: ESP32 firmware that samples sensors, determines parking
  state, and exposes that state for transmission.
- **Connectivity Layer**: The transport used to carry device state to the
  backend (protocol pending — see `ADR-0005`).
- **Platform/Backend Layer**: Ingests device updates, maintains slot state, and
  serves availability to clients.
- **Persistence Layer**: Stores current state and occupancy history.
- **Presentation Layer**: The dashboard that presents availability to users.

Each layer communicates through well-defined boundaries. This mirrors the
repository layout: `firmware/`, `server/`, `client/`.

## Alternatives Considered

- **Monolithic firmware + single web app**: rejected — couples real-time device
  handling with UI and makes independent testing harder.
- **Pushing state directly from device to UI**: rejected — creates fragile
  point-to-point coupling and no durable record.

## Consequences

### Positive

- Clear ownership of each concern (firmware, backend, dashboard).
- Layers can be developed and tested independently.
- Failure in one layer (e.g. a device offline) is isolated.
- Matches the existing repository directory structure.

### Negative

- More moving parts than a monolithic approach.
- Requires a well-defined communication contract between layers
  (`../architecture/communication.md`).

## Validation

- Repository is organized into `firmware/`, `server/`, and `client/` directories.
- Architecture is reflected in `../architecture/system-architecture.md`.

## Related Documents

- `../architecture/system-architecture.md`
- `../architecture/firmware-architecture.md`
- `../architecture/backend-architecture.md`
- `../product/VISION.md`
