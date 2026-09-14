/**
 * Everything the UI concludes from the wire contract, as pure functions.
 *
 * Kept out of components so the interesting judgements — what counts as full,
 * what counts as stale, when a section is trustworthy — are stated once and
 * tested directly instead of being re-derived in every card.
 *
 * Derived shapes use camelCase; anything snake_case in here came off the wire
 * unchanged (see `contract.ts`).
 */
import {
  type NodeStatus,
  type SectionState,
  type Slot,
  type SlotState,
  sectionKeyOf,
} from "./contract";

/**
 * A node republishes its full snapshot every 30 s ± 10 %, so 33 s is the worst
 * case for a healthy section. Past 45 s the last accepted snapshot is old
 * enough that the operator should be told rather than shown a confident number.
 *
 * A section is never *declared* offline from silence: only a retained
 * `node_status offline` frame (the node's MQTT will) can do that, and the
 * backend clears the section's state when it arrives.
 */
export const STALE_AFTER_MS = 45_000;

export const SLOT_STATES = ["free", "occupied", "error"] as const;

export type SectionHealth =
  /** Reporting, and the last snapshot is recent. */
  | "live"
  /** Reporting, but the last accepted snapshot is older than `STALE_AFTER_MS`. */
  | "stale"
  /** The node's will fired: the backend cleared this section's state. */
  | "offline"
  /** Known to exist (a `node_status` arrived) but no snapshot yet. */
  | "unreported";

export interface SlotTotals {
  free: number;
  occupied: number;
  error: number;
  /** Slots the section is known to carry, including faulty ones. */
  total: number;
}

/** One section as the UI needs it: wire state plus every conclusion about it. */
export interface SectionView {
  key: string;
  site: string;
  section: string;
  /** `null` once a node goes offline, mirroring the backend clearing state. */
  state: SectionState | null;
  /** Last `node_status` frame seen this session, if any. */
  nodeStatus: NodeStatus | null;
  health: SectionHealth;
  totals: SlotTotals;
  /** Age of the last accepted snapshot, or `null` when there is none. */
  ageMs: number | null;
  /**
   * Occupied share of the slots whose state is actually known, i.e. excluding
   * faulty sensors. `null` when nothing is known. A faulty slot is not free.
   */
  occupancy: number | null;
}

export interface SiteTotals extends SlotTotals {
  sections: number;
  live: number;
  stale: number;
  offline: number;
  unreported: number;
  occupancy: number | null;
}

export function countSlots(slots: readonly Slot[]): SlotTotals {
  const totals: SlotTotals = { free: 0, occupied: 0, error: 0, total: 0 };

  for (const slot of slots) {
    totals[slot.state] += 1;
    totals.total += 1;
  }

  return totals;
}

/**
 * Occupied share of the *known* slots. Faulty sensors are excluded from the
 * denominator rather than counted as free, because a broken sensor is not an
 * available bay and pretending otherwise is the one error an availability board
 * must not make.
 */
export function occupancyOf(totals: SlotTotals): number | null {
  const known = totals.free + totals.occupied;

  return known === 0 ? null : totals.occupied / known;
}

export function sectionHealth(
  state: SectionState | null,
  nodeStatus: NodeStatus | null,
  now: number,
): SectionHealth {
  if (nodeStatus === "offline") return "offline";
  if (!state) return "unreported";

  return now - state.server_ts_ms > STALE_AFTER_MS ? "stale" : "live";
}

export function toSectionView(
  key: string,
  state: SectionState | null,
  nodeStatus: NodeStatus | null,
  now: number,
  fallbackId?: { site: string; section: string },
): SectionView {
  const identity = state ?? fallbackId ?? splitSectionKey(key);
  const totals = state
    ? countSlots(state.slots)
    : { free: 0, occupied: 0, error: 0, total: 0 };

  return {
    key,
    site: identity.site,
    section: identity.section,
    state,
    nodeStatus,
    health: sectionHealth(state, nodeStatus, now),
    totals,
    ageMs: state ? Math.max(0, now - state.server_ts_ms) : null,
    occupancy: state ? occupancyOf(totals) : null,
  };
}

/** Inverse of `sectionKey`, for keys held without their state (offline nodes). */
export function splitSectionKey(key: string): {
  site: string;
  section: string;
} {
  const separator = key.indexOf("/");

  return separator === -1
    ? { site: key, section: "" }
    : { site: key.slice(0, separator), section: key.slice(separator + 1) };
}

export function summariseSite(views: readonly SectionView[]): SiteTotals {
  const totals: SiteTotals = {
    free: 0,
    occupied: 0,
    error: 0,
    total: 0,
    sections: views.length,
    live: 0,
    stale: 0,
    offline: 0,
    unreported: 0,
    occupancy: null,
  };

  for (const view of views) {
    totals.free += view.totals.free;
    totals.occupied += view.totals.occupied;
    totals.error += view.totals.error;
    totals.total += view.totals.total;
    totals[view.health] += 1;
  }

  totals.occupancy = occupancyOf(totals);

  return totals;
}

/**
 * Board order: site first, then section, comparing numerically where the label
 * is numeric so `A2` sorts before `A10`.
 */
export function compareSections(a: SectionView, b: SectionView): number {
  return (
    a.site.localeCompare(b.site, undefined, { numeric: true }) ||
    a.section.localeCompare(b.section, undefined, { numeric: true })
  );
}

/** Slot order within a section, so `A-2` precedes `A-10`. */
export function compareSlots(a: Slot, b: Slot): number {
  return a.id.localeCompare(b.id, undefined, { numeric: true });
}

/**
 * Slot-level diff between two accepted snapshots of one section, which is what
 * an operator reads as "something happened". Slots that vanished or appeared
 * are ignored: the backend enforces a stable slot count per section, so that
 * can only mean a re-provisioned node, and the next snapshot is authoritative.
 */
export interface SlotTransition {
  slotId: string;
  from: SlotState;
  to: SlotState;
}

export function diffSlots(
  previous: SectionState | null,
  next: SectionState,
): SlotTransition[] {
  if (!previous) return [];

  const before = new Map(previous.slots.map((slot) => [slot.id, slot.state]));
  const transitions: SlotTransition[] = [];

  for (const slot of next.slots) {
    const from = before.get(slot.id);

    if (from !== undefined && from !== slot.state) {
      transitions.push({ slotId: slot.id, from, to: slot.state });
    }
  }

  return transitions;
}

/** Distinct sites present in a section list, in board order. */
export function sitesOf(views: readonly SectionView[]): string[] {
  return [...new Set(views.map((view) => view.site))].sort((a, b) =>
    a.localeCompare(b, undefined, { numeric: true }),
  );
}

export function sectionViewKey(state: SectionState): string {
  return sectionKeyOf(state);
}
