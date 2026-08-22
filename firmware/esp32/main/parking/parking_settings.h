#pragma once

/**
 * Algorithm and system tuning knobs (non-hardware).
 *
 * Spec: docs/specs/architecture/firmware-architecture.md (§ Configuration).
 *
 * What lives where:
 *   parking_config.h/.c  - hardware + per-slot wiring (slot count, GPIOs,
 *                          per-slot thresholds; thresholds depend on sensor
 *                          mounting height, so they stay with the hardware)
 *   parking_settings.h   - algorithm/system defaults (this file)
 *   ultrasonic.h         - driver-owned measurement limits (2-400 cm, timeout)
 */

/** Period between parking lot scans in milliseconds. */
#define PARKING_SCAN_INTERVAL_MS 1000

/**
 * EMA smoothing factor for distance stability, in the range (0..1].
 *
 * Higher values react faster to real changes; lower values smooth more.
 * With a 1 s scan period, 0.3 settles ~90 % within about six scans — fast
 * enough for a car parking, slow enough to swallow echo jitter.
 */
#define PARKING_FILTER_ALPHA 0.3f

/**
 * Consecutive readings of the same candidate state required before an
 * occupancy transition fires. Confirmation counting is signal-processing
 * policy shared by all identical sensors, hence global rather than per-slot.
 */
#define PARKING_OCCUPIED_CONFIRMATION_COUNT 2
#define PARKING_FREE_CONFIRMATION_COUNT 2

/** Parking scan task: 4 KiB stack (scan + logs + filter state headroom). */
#define PARKING_TASK_STACK_SIZE 4096

/** Priority above idle/log workers, below critical system tasks. */
#define PARKING_TASK_PRIORITY 5

/** FreeRTOS name of the parking task (shown in traces / stack dumps). */
#define PARKING_TASK_NAME "parking_task"
