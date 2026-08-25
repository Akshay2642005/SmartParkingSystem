# Verifications

Verification records are stored here when tests are performed.

Each record should use `../templates/VERIFICATION.md` and contain the objective,
preconditions, environment, procedure, expected/actual results, evidence, and a
PASS/FAIL/BLOCKED result.

No fake successful reports are invented; this directory is empty until a real
verification is performed.

## Planned Verification Records

- `wokwi-boot.md` — firmware boots in Wokwi.
- `sensor-integration.md` — sensor driver reflects occupancy. (recorded, PASS)
- `device-backend.md` — device state reaches the backend. (recorded, PASS)
- `end-to-end.md` — full device → backend → dashboard path.

Also recorded here:

- `m2-completion-criteria.md` — PROMPT.md local-subsystem completion
  checklist audit (M2 close-out). PASS.

## Related Documents

- `../TEST_PLAN.md`
- `../../templates/VERIFICATION.md`