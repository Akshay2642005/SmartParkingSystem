# Smart Parking System — Product Vision

Status: **Current** (M0 — Specification baseline)

> A real-time IoT parking management system that detects parking-slot occupancy
> using ESP32-connected sensors, reports parking state to a backend platform,
> persists parking information, and exposes current parking availability to users.

This statement is a working concept and is refined as the project evolves.

---

## Problem

Drivers and parking managers lack reliable, near-real-time visibility into which
parking slots are currently occupied. Manually tracked or unobserved parking
areas lead to wasted time, congestion, and inefficient use of parking capacity.
There is no lightweight, self-contained system that connects edge sensors, a
backend, and a user-facing availability view for this problem domain.

## Target Users

- **Parking managers / operators** — need to see current occupancy and, later,
  historical occupancy of a parking area.
- **Drivers** — need to know which slots are available before or on arrival.
- **Developers / maintainers** — need clear specifications to build and extend
  the system.

The MVP primarily targets **parking managers/operators** and **developers**.
Driver-facing functionality may follow.

## Product Vision

Build a system in which ESP32-based edge devices, each connected to one or more
parking-slot sensors, continuously determine slot occupancy, report that state
to a backend platform over a network, persist the information, and surface
current parking availability through a dashboard.

## Goals

- Detect and track parking-slot occupancy using ESP32-connected sensors.
- Reliably report device and slot state to a backend.
- Persist current state and a history of occupancy transitions.
- Expose current parking availability to users via a dashboard.
- Keep device firmware, backend, and dashboard cleanly separated and testable.

## Success Criteria

- A slot's occupancy state is determined by sensor data and maintained over time.
- Device state reaches the backend and is persisted.
- The dashboard displays occupied vs. available slots accurately.
- The system recovers from temporary device/network failures.
- Planned functionality is specified and verified before release.

Numeric targets (e.g. latency, accuracy, uptime) are **Pending Decision** and are
not invented here.

## System Boundaries

The system covers:

- ESP32-based edge devices and their sensors.
- Device-to-backend connectivity.
- Backend ingestion, state management, and persistence.
- A dashboard for availability.

Out of scope for the MVP (see `SCOPE.md`): payment processing, license-plate
recognition, computer vision, physical gate hardware, and production hardware
certification.

## High-Level User Experience

1. A sensor detects a change at a parking slot.
2. The ESP32 updates the slot state.
3. The device sends the state to the backend.
4. The backend persists the state and history.
5. The dashboard reflects the updated availability to the user.

## Future Direction

- Historical occupancy reporting and analytics.
- Driver-facing live availability.
- Multiple sites / parking lots.
- Device fleet management and OTA updates.
- Payment and reservation integrations (if required by product direction).

---

## Related Documents

- `SCOPE.md`
- `REQUIREMENTS.md`
- `../architecture/system-architecture.md`
- `../planning/PLAN.md`
