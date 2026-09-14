/**
 * Value formatting for a live console.
 *
 * Two rules shape everything here:
 *
 * - **No locale surprises.** Durations and clock times are formatted with fixed
 *   ASCII patterns rather than `Intl`, so a value rendered on the server and
 *   re-rendered in the browser is byte-identical and React never reports a
 *   hydration mismatch. `Intl` is used only where the locale is genuinely the
 *   point (thousands separators in big counts).
 * - **Uptime milliseconds are not wall clock.** `changed_ms` is the node's own
 *   monotonic uptime, so it is only ever rendered as an elapsed duration —
 *   never as a time of day.
 */

const SECOND = 1000;
const MINUTE = 60 * SECOND;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/** Compact age of an event: `now`, `12s`, `4m 20s`, `3h 05m`, `2d 04h`. */
export function formatAge(ms: number): string {
  if (!Number.isFinite(ms)) return "—";
  if (ms < 0) return "now";
  if (ms < SECOND) return "now";

  if (ms < MINUTE) return `${Math.floor(ms / SECOND)}s`;

  if (ms < HOUR) {
    const minutes = Math.floor(ms / MINUTE);
    const seconds = Math.floor((ms % MINUTE) / SECOND);

    return `${minutes}m ${pad(seconds)}s`;
  }

  if (ms < DAY) {
    const hours = Math.floor(ms / HOUR);
    const minutes = Math.floor((ms % HOUR) / MINUTE);

    return `${hours}h ${pad(minutes)}m`;
  }

  const days = Math.floor(ms / DAY);
  const hours = Math.floor((ms % DAY) / HOUR);

  return `${days}d ${pad(hours)}h`;
}

/** `formatAge` with the direction spelled out, for prose and tooltips. */
export function formatAgo(ms: number): string {
  const age = formatAge(ms);

  return age === "now" ? "just now" : `${age} ago`;
}

/** Service uptime from `GET /status`, which reports whole seconds. */
export function formatUptime(seconds: number): string {
  return formatAge(seconds * SECOND);
}

/**
 * Wall-clock time of day, UTC, 24 h. UTC because `server_ts_ms` is the
 * backend's clock and a console comparing it against device logs should not
 * silently shift it into the viewer's zone.
 */
export function formatClockUtc(epochMs: number): string {
  const date = new Date(epochMs);

  if (Number.isNaN(date.getTime())) return "—";

  return `${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(
    date.getUTCSeconds(),
  )}Z`;
}

/** Full UTC timestamp for tooltips and table cells: `2026-08-27 14:03:11Z`. */
export function formatTimestampUtc(epochMs: number): string {
  const date = new Date(epochMs);

  if (Number.isNaN(date.getTime())) return "—";

  return (
    `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-` +
    `${pad(date.getUTCDate())} ${formatClockUtc(epochMs)}`
  );
}

/** Percentage with no decimals: occupancy is never precise enough for more. */
export function formatPercent(ratio: number | null): string {
  return ratio === null ? "—" : `${Math.round(ratio * 100)}%`;
}

/** Counts big enough to need grouping (`seq` can run into the millions). */
export function formatCount(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}

function pad(value: number): string {
  return value.toString().padStart(2, "0");
}
