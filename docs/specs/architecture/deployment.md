# Deployment

Status: **Planned / Pending Decision** (no cloud provider chosen)

This document describes the intended deployment topology across development,
staging, and production environments.

## Development

```text
Development
    ↓
Wokwi
```

Firmware is developed and verified in Wokwi simulation
(`../decisions/ADR-0003-wokwi.md`). No cloud infrastructure is required.

## Staging

```text
Staging
    ↓
Simulated/physical devices
    ↓
Cloud backend
```

Staging exercises the full path with simulated or physical devices against a
deployed backend.

## Production

```text
Production
    ↓
Physical ESP32 devices
    ↓
Network
    ↓
Cloud backend
```

Physical devices connect over a network to the cloud backend, which serves the
dashboard.

## Decisions Pending

- **Cloud provider**: Pending Decision. No provider is assumed.
- Backend hosting model: Pending Decision.
- Database hosting: Pending Decision.
- Dashboard hosting: Pending Decision.
- Network/topology details: Pending Decision.

## Related Documents

- `../decisions/ADR-0003-wokwi.md`
- `../decisions/ADR-0006-backend.md`
- `system-architecture.md`
- `../security/SECURITY_PLAN.md`
- `../planning/PLAN.md` (M6, M7)