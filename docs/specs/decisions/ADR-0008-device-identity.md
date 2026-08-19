# ADR-0008: Device Identity

Status: Proposed

## Context

The backend must distinguish state updates from different devices and associate
them with parking slots. No identity model is implemented yet.

## Problem

How should individual devices be identified, and how should identity relate to
slots?

## Decision

**Pending Decision** on the exact identity scheme. The candidate identity
concepts are recorded here and must be separated from authentication and
authorization.

## Identity Concepts

```text
Device ID      → uniquely identifies a physical/edge device
Slot ID        → uniquely identifies a parking slot
Installation ID→ links a device to a slot/lot deployment
Site ID        → identifies a physical location (future)
```

## Separation of Concerns

```text
Identity          → who/what is this (stable identifier)
Authentication    → proving the identity (cryptographic proof)
Authorization     → what the identity is allowed to do
```

Defining identity does not imply implementing authentication. Authentication is
scoped separately in `../security/SECURITY_PLAN.md`.

## Decision Details

- The firmware currently has **no identity model** and no persistence of device
  credentials.
- A scheme (device ↔ slot association) must be chosen before the device platform
  milestone (M3) and backend ingestion (M4).

## Consequences

### Positive

- Clear mapping of devices to slots enables correct state attribution.

### Negative

- A late change to the identity scheme may require rework of firmware and
  backend.

## Validation

- Database model reflects the chosen identity scheme (`../architecture/database.md`).
- Backend ingestion correctly attributes updates to slots (`FR-031`).

## Related Documents

- `../architecture/database.md`
- `../security/SECURITY_PLAN.md`
- `../planning/PLAN.md` (M3, M4)