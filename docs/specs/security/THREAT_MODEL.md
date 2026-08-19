# Threat Model

Status: **Planned** (initial threat identification; mitigations are not yet
implemented)

This document identifies threats to the Smart Parking System. Mitigations are
only listed as existing if implemented. Most are `Planned` or `Pending Decision`.

## Device

- Firmware extraction
- Credential extraction
- Device impersonation
- Firmware tampering
- Physical access
- Sensor spoofing

## Network

- Man-in-the-Middle (MITM)
- Replay
- Eavesdropping
- Message injection
- Denial of Service (DoS)

## Backend

- Unauthorized devices
- API abuse
- Privilege escalation
- Data manipulation

## Threat Table

| Threat | Asset | Impact | Likelihood | Mitigation | Status |
| ------ | ----- | ------ | ---------- | ---------- | ------ |
| Firmware extraction | Device firmware | High | Medium | Code/credential protection | Pending Decision |
| Credential extraction | Device credentials | High | Medium | Secure storage | Pending Decision |
| Device impersonation | Backend integrity | High | Medium | Device authentication | Pending Decision |
| Firmware tampering | Device behavior | High | Medium | Firmware signing | Pending Decision |
| Physical access | Device | Medium | Medium | Tamper resistance | Pending Decision |
| Sensor spoofing | Occupancy data | Medium | Medium | Input validation / sanity checks | Pending Decision |
| MITM | Data in transit | High | Medium | TLS | Pending Decision |
| Replay | Data integrity | Medium | Medium | Nonce/timestamp + idempotency | Pending Decision |
| Eavesdropping | Data confidentiality | Medium | Medium | TLS | Pending Decision |
| Message injection | Data integrity | Medium | Medium | Authentication | Pending Decision |
| DoS | Availability | Medium | Medium | Rate limiting | Pending Decision |
| Unauthorized devices | Backend | High | Medium | Device authentication | Pending Decision |
| API abuse | Backend | Medium | Medium | Rate limiting / authz | Pending Decision |
| Privilege escalation | Backend | High | Medium | Authorization | Pending Decision |
| Data manipulation | Persisted state | High | Medium | Access control + integrity | Pending Decision |

## Related Documents

- `SECURITY_PLAN.md`
- `DEVICE_SECURITY.md`
- `../product/REQUIREMENTS.md` (NFR-006)