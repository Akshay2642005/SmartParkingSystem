# Smart Parking System — Specifications

This directory is the **source of truth for the Smart Parking System specification**.
It describes what the system does, why it is built this way, how it is designed,
how it will be implemented and verified, and how it is secured.

> **Status of this document set:** `Current` (M0 — Specification). The repository
> is still in its initial scaffolding state. Where a decision has not been made,
> the relevant section is explicitly marked `Pending Decision`. Nothing is claimed
> to be implemented unless it exists in the repository source code.

---

## Purpose

The specifications exist so that a developer joining the project can understand
the system **without reverse-engineering the architecture from source code**.

They answer:

- **WHAT** the system does — `product/`
- **WHY** architectural decisions were made — `decisions/`
- **HOW** the system is designed — `architecture/`
- **WHEN** and in what order work happens — `planning/`
- **HOW** we prove it works — `quality/`
- **HOW** we protect it — `security/`
- **WHAT** has been superseded — `archive/`
- **REUSABLE** specification templates — `templates/`

---

## Spec-First Workflow

Specification flows down into implementation and verification:

```text
Requirements
     ↓
Architecture
     ↓
Architectural Decisions
     ↓
Planning
     ↓
Implementation
     ↓
Verification
     ↓
Release
```

A requirement is first **written**, then **designed**, then an architectural
**decision** is recorded, then it is **planned**, then **implemented**, and
finally **verified** against the requirement.

---

## Document Organization

```text
specs/
├── product/        WHAT we build and why
├── decisions/      WHY we build it this way
├── architecture/   HOW the system is designed
├── planning/       WHEN and in what order
├── quality/        HOW we prove it works
├── security/       HOW we protect it
├── archive/        WHAT has been superseded
└── templates/      REUSABLE specification templates
```

## Source-of-Truth Rules

```text
Specification   → describes intended behavior
Source Code     → implements behavior
Tests           → verify behavior
```

- Specifications describe **intended** behavior and must not pretend planned
  functionality already exists.
- Source code is the authority for **what is currently implemented**.
- Every specification uses explicit status labels:
  `Current`, `Planned`, `Proposed`, `Experimental`, `Deferred`, `Superseded`,
  `Pending Decision`.
- If a specification and the code disagree, the **code wins** for current state
  and the specification must be corrected or marked accordingly.

---

## How To ...

### Add an ADR

1. Copy `templates/ADR.md` into `decisions/`.
2. Name it `ADR-NNNN-<kebab-case-slug>.md` where `NNNN` is the next sequential
   number.
3. Fill in Context, Problem, Decision, Alternatives, Consequences, Validation,
   and Related Documents.
4. Set a status: `Proposed`, `Accepted`, `Superseded`, or `Rejected`.
5. Link the ADR from this README and from any architecture document it affects.

### Update Architecture

- Edit the relevant file under `architecture/`.
- When an architecture document changes because of an ADR, reference the ADR.
- When an ADR is accepted, update every dependent document so the system stays
  internally consistent (see Consistency Rules below).

### Update Planning State

- Update `planning/STATUS.yaml` for the current phase and task status.
- Update `planning/PHASES.yaml` when a phase transitions between
  `planned` / `active` / `completed`.
- Update `planning/SESSION.yaml` at the start of each working session so a
  future agent can resume work immediately.
- Update `planning/RELEASES.yaml` only when a release is actually planned or cut.

### Run Verification

- Use `templates/VERIFICATION.md` to record each verification.
- Store completed records under `quality/verifications/`.
- Store audit findings under `quality/audits/`.
- Record defects in `quality/BUGS.yaml`.

---

## Consistency Rules

Specifications must be internally consistent. For example:

- If an ADR (e.g. `ADR-0005`) chooses a communication protocol, then
  `architecture/communication.md`, `architecture/system-architecture.md`, and
  `planning/PLAN.md` must reflect that protocol.
- When a decision changes, update dependent documents and move the superseded
  decision to `archive/decisions.md`.
- Avoid contradictions between documents.

## Status Rules

Use explicit status labels — not vague wording:

- `Current`
- `Planned`
- `Proposed`
- `Experimental`
- `Deferred`
- `Superseded`
- `Pending Decision`

Avoid `maybe`, `probably`, `we might`, `I think`, unless explicitly documenting
an unresolved discussion.

---

## Document Index

| Document | Purpose |
| -------- | ------- |
| `README.md` | Master index and workflow guide (this file) |
| `product/VISION.md` | Product vision, users, goals, success criteria |
| `product/SCOPE.md` | In-scope / out-of-scope, MVP and deferred features |
| `product/REQUIREMENTS.md` | Functional and non-functional requirements |
| `product/GLOSSARY.yaml` | Structured domain glossary |
| `product/snapshots/v0.1.0.md` | Initial specification baseline snapshot |
| `decisions/ADR-0001-architecture.md` | System architecture layering |
| `decisions/ADR-0002-esp-idf.md` | ESP-IDF as the firmware framework |
| `decisions/ADR-0003-wokwi.md` | Wokwi as the simulation environment |
| `decisions/ADR-0004-sensor-selection.md` | Parking sensor selection |
| `decisions/ADR-0005-device-communication.md` | Device ↔ backend communication |
| `decisions/ADR-0006-backend.md` | Backend technology and boundaries |
| `decisions/ADR-0007-parking-state-model.md` | Parking state machine |
| `decisions/ADR-0008-device-identity.md` | Device identification model |
| `decisions/ADR-0009-real-time-updates.md` | Real-time dashboard updates |
| `architecture/tech-stack.md` | Technology inventory |
| `architecture/system-architecture.md` | System context, containers, data flow |
| `architecture/firmware-architecture.md` | ESP32 firmware architecture |
| `architecture/hardware-architecture.md` | Hardware topology and mapping |
| `architecture/communication.md` | Device communication contract |
| `architecture/backend-architecture.md` | Backend API and services |
| `architecture/database.md` | Conceptual data model |
| `architecture/deployment.md` | Deployment topology |
| `planning/PLAN.md` | Milestone-based implementation plan |
| `planning/STATUS.yaml` | Machine-readable current status |
| `planning/PHASES.yaml` | Machine-readable project phases |
| `planning/RELEASES.yaml` | Release tracking |
| `planning/SESSION.yaml` | Current development session |
| `quality/TEST_PLAN.md` | Layered testing strategy |
| `quality/BUGS.yaml` | Machine-readable bug tracker |
| `quality/audits/` | Completed audit records |
| `quality/verifications/` | Completed verification records |
| `security/SECURITY_PLAN.md` | Security plan |
| `security/THREAT_MODEL.md` | IoT threat model |
| `security/DEVICE_SECURITY.md` | Device security strategy |
| `archive/decisions.md` | Superseded decisions archive |
| `templates/ADR.md` | Reusable ADR template |
| `templates/DESIGN.md` | Reusable design document template |
| `templates/VERIFICATION.md` | Reusable verification record template |
