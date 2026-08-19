# Device Security

Status: **Planned** (strategy; production security mechanisms are not required in
Wokwi simulation)

This document describes device security across development, simulation, and
production environments.

## Covered Areas

- Device identity
- Credentials
- Secure storage
- TLS
- Certificate strategy
- OTA
- Firmware signing
- Boot security
- Production provisioning

## Environments

### Development

- No production security mechanisms are enforced.
- Focus is on functional correctness and local tooling.

### Simulation (Wokwi)

- Production security mechanisms are **not required** in Wokwi
  (`../decisions/ADR-0003-wokwi.md`).
- Simulated credentials are placeholders only and must not be treated as secure.

### Production

- Device identity: per `../decisions/ADR-0008-device-identity.md`.
- Credentials: stored securely (mechanism Pending Decision).
- Secure storage: Pending Decision.
- TLS: for device-to-backend transport, Pending Decision.
- Certificate strategy: Pending Decision.
- OTA: Pending Decision.
- Firmware signing: Pending Decision.
- Boot security: Pending Decision.
- Production provisioning: Pending Decision.

## Principle

Do not require production security mechanisms in Wokwi. Security mechanisms are
scoped to the environment in which they apply.

## Related Documents

- `SECURITY_PLAN.md`
- `THREAT_MODEL.md`
- `../decisions/ADR-0003-wokwi.md`
- `../decisions/ADR-0008-device-identity.md`
- `../architecture/deployment.md`