/**
 * The live view of parking state, as a pure reducer.
 *
 * `ADR-0009` gives the dashboard a snapshot-then-deltas contract, and the whole
 * point of that contract is that the client needs no reconciliation logic: a
 * snapshot replaces everything, an update replaces one section, and a
 * reconnect starts over with a fresh snapshot. So all of the interesting
 * behaviour is here, in functions that take state and an event and return new
 * state — no React, no sockets, no clock of their own.
 *
 * The rules this reducer keeps, and why:
 *
 * - **A snapshot is the whole truth.** Sections absent from it no longer have
 *   state in the backend, so they are dropped rather than left on the board
 *   showing numbers nobody stands behind. The one exception is a section we
 *   have seen go offline: its key is kept with `state: null` so the operator
 *   still sees that the bay block exists and its node is dead.
 * - **Presence implies liveness.** The backend clears a section's state when a
 *   retained `offline` arrives, so a section that appears in a snapshot or an
 *   update necessarily has a live node. That inference is what lets the board
 *   show node health without waiting for a `node_status` frame.
 * - **`offline` clears state.** Mirroring the backend exactly, because the
 *   alternative — keeping the last snapshot of a dead node — is how an
 *   availability board ends up sending drivers to occupied bays.
 * - **Updates are applied blind.** No `seq` comparison: the backend has
 *   already rejected stale and out-of-order publishes, snapshots are complete
 *   rather than deltas, and a node reboot legitimately restarts `seq` at 1.
 *   Re-checking `seq` here would only re-implement a guard we would then have
 *   to keep in step.
 * - **The activity feed is edited, not exhaustive.** Nodes republish every
 *   30 s whether or not anything changed, so logging every frame would bury
 *   the signal. Slot transitions, node status changes, connection changes, and
 *   errors are logged; unchanged refreshes only move `lastFrameAt`.
 */
import type {
  ErrorCode,
  NodeStatus,
  SectionState,
  ServerEvent,
  SlotState,
} from "@/lib/parking/contract";
import { sectionKey, sectionKeyOf } from "@/lib/parking/contract";
import { countSlots, diffSlots } from "@/lib/parking/derive";

/** How the socket is doing. Drives the header pill and the REST fallback. */
export type ConnectionPhase =
  /** No socket yet (server render, or before mount). */
  | "idle"
  /** First connection attempt in flight. */
  | "connecting"
  /** Connected; a snapshot has been requested or received. */
  | "open"
  /** Dropped, and retrying with backoff. */
  | "reconnecting"
  /** Deliberately closed (unmount, or the browser went offline). */
  | "stopped";

export type LiveSource = "none" | "socket" | "rest";

export interface SectionEntry {
  /** `null` means the backend holds no state: never reported, or node offline. */
  state: SectionState | null;
  nodeStatus: NodeStatus | null;
}

export type ActivityEntry = { id: number; at: number } & (
  | {
      kind: "slot";
      site: string;
      section: string;
      slotId: string;
      from: SlotState;
      to: SlotState;
    }
  | { kind: "node"; site: string; section: string; status: NodeStatus }
  | { kind: "snapshot"; sections: number; source: LiveSource }
  | { kind: "connection"; phase: ConnectionPhase; detail?: string }
  | { kind: "error"; code: ErrorCode | "transport"; message: string }
);

export type ActivityKind = ActivityEntry["kind"];

/** One point on the session occupancy chart. */
export interface OccupancySample {
  t: number;
  free: number;
  occupied: number;
  error: number;
}

export interface ConnectionState {
  phase: ConnectionPhase;
  /** When the phase was entered, for "reconnecting for 12s". */
  since: number | null;
  /** Consecutive failed attempts; resets on a successful open. */
  attempt: number;
  lastError: string | null;
}

export interface LiveState {
  entries: Record<string, SectionEntry>;
  activity: ActivityEntry[];
  timeline: OccupancySample[];
  connection: ConnectionState;
  /** Last time any frame arrived, socket or REST. Liveness for the header. */
  lastFrameAt: number | null;
  lastSnapshotAt: number | null;
  source: LiveSource;
  nextActivityId: number;
}

/**
 * Feed and chart caps. Both are session-local: the backend keeps no history
 * (`database.md` stores current state only), so these buffers are the entire
 * historical record and are deliberately small enough to never be a memory
 * concern on a wall display left open for days.
 */
export const ACTIVITY_LIMIT = 300;
export const TIMELINE_LIMIT = 240;

export function initialLiveState(): LiveState {
  return {
    entries: {},
    activity: [],
    timeline: [],
    connection: { phase: "idle", since: null, attempt: 0, lastError: null },
    lastFrameAt: null,
    lastSnapshotAt: null,
    source: "none",
    nextActivityId: 1,
  };
}

export function applyServerEvent(
  state: LiveState,
  event: ServerEvent,
  now: number,
): LiveState {
  switch (event.type) {
    case "snapshot":
      return withTimeline(
        {
          ...state,
          entries: rebuildEntries(state.entries, event.sections),
          lastFrameAt: now,
          lastSnapshotAt: now,
          source: "socket",
          ...log(state, now, {
            kind: "snapshot",
            sections: event.sections.length,
            source: "socket",
          }),
        },
        now,
      );

    case "update": {
      const key = sectionKeyOf(event.section);
      const previous = state.entries[key]?.state ?? null;
      const transitions = diffSlots(previous, event.section);

      return withTimeline(
        {
          ...state,
          entries: {
            ...state.entries,
            [key]: { state: event.section, nodeStatus: "online" },
          },
          lastFrameAt: now,
          source: "socket",
          ...logMany(
            state,
            now,
            transitions.map((transition) => ({
              kind: "slot" as const,
              site: event.section.site,
              section: event.section.section,
              slotId: transition.slotId,
              from: transition.from,
              to: transition.to,
            })),
          ),
        },
        now,
      );
    }

    case "node_status": {
      const key = sectionKey(event.site, event.section);
      const known = state.entries[key];
      const unchanged = known?.nodeStatus === event.status;

      return withTimeline(
        {
          ...state,
          entries: {
            ...state.entries,
            [key]: {
              // Offline clears state, exactly as the backend does, so the next
              // post-reboot snapshot (seq restarted at 1) is what refills it.
              state: event.status === "offline" ? null : (known?.state ?? null),
              nodeStatus: event.status,
            },
          },
          lastFrameAt: now,
          source: "socket",
          ...(unchanged
            ? {}
            : log(state, now, {
                kind: "node",
                site: event.site,
                section: event.section,
                status: event.status,
              })),
        },
        now,
      );
    }

    case "error":
      return {
        ...state,
        lastFrameAt: now,
        connection: { ...state.connection, lastError: event.message },
        ...log(state, now, {
          kind: "error",
          code: event.code,
          message: event.message,
        }),
      };
  }
}

/**
 * Apply a REST section list: the server-rendered first paint, and the polling
 * fallback used while the socket is down. Identical handling to a `snapshot`
 * frame because `GET {prefix}/sections` returns the same `SectionState` shape —
 * the two are the same truth over different transports.
 */
export function applyRestSnapshot(
  state: LiveState,
  sections: readonly SectionState[],
  now: number,
): LiveState {
  return withTimeline(
    {
      ...state,
      entries: rebuildEntries(state.entries, sections),
      lastFrameAt: now,
      lastSnapshotAt: now,
      // A socket that is already open owns the truth; a poll landing during
      // that window must not relabel the source and make the UI claim it is
      // falling back.
      source: state.connection.phase === "open" ? state.source : "rest",
      ...log(state, now, {
        kind: "snapshot",
        sections: sections.length,
        source: "rest",
      }),
    },
    now,
  );
}

/**
 * A frame the transport could not decode, or that did not match the event
 * contract. Logged with the reserved `transport` code so the feed distinguishes
 * "the backend reported an error" from "we could not understand the backend" —
 * the second one means the two deployments disagree about the contract.
 */
export function applyProtocolError(
  state: LiveState,
  message: string,
  now: number,
): LiveState {
  return {
    ...state,
    connection: { ...state.connection, lastError: message },
    ...log(state, now, { kind: "error", code: "transport", message }),
  };
}

export function applyConnectionPhase(
  state: LiveState,
  phase: ConnectionPhase,
  now: number,
  options: { attempt?: number; detail?: string } = {},
): LiveState {
  if (state.connection.phase === phase && options.detail === undefined) {
    return state;
  }

  return {
    ...state,
    connection: {
      phase,
      since: now,
      attempt: options.attempt ?? (phase === "open" ? 0 : state.connection.attempt),
      lastError: options.detail ?? (phase === "open" ? null : state.connection.lastError),
    },
    ...log(state, now, { kind: "connection", phase, detail: options.detail }),
  };
}

/**
 * Rebuild the section map from an authoritative list. Sections in the list are
 * live by definition; sections we last saw go offline are kept as empty entries
 * so a dead node stays visible; anything else is forgotten.
 */
function rebuildEntries(
  previous: Record<string, SectionEntry>,
  sections: readonly SectionState[],
): Record<string, SectionEntry> {
  const entries: Record<string, SectionEntry> = {};

  for (const [key, entry] of Object.entries(previous)) {
    if (entry.nodeStatus === "offline") {
      entries[key] = { state: null, nodeStatus: "offline" };
    }
  }

  for (const state of sections) {
    entries[sectionKeyOf(state)] = { state, nodeStatus: "online" };
  }

  return entries;
}

// Distribute Omit over the union so each variant still discriminates on `kind`.
type ActivityInput = ActivityEntry extends infer E
  ? E extends { id: number; at: number }
    ? Omit<E, "id" | "at">
    : never
  : never;

function log(
  state: LiveState,
  now: number,
  entry: ActivityInput,
): Pick<LiveState, "activity" | "nextActivityId"> {
  return logMany(state, now, [entry]);
}

function logMany(
  state: LiveState,
  now: number,
  inputs: readonly ActivityInput[],
): Pick<LiveState, "activity" | "nextActivityId"> {
  if (inputs.length === 0) {
    return { activity: state.activity, nextActivityId: state.nextActivityId };
  }

  // Newest first: the feed is read from the top and the cap drops the tail.
  const added = inputs.map((input, index) => ({
    ...input,
    id: state.nextActivityId + index,
    at: now,
  })) as ActivityEntry[];

  return {
    activity: [...added.reverse(), ...state.activity].slice(0, ACTIVITY_LIMIT),
    nextActivityId: state.nextActivityId + inputs.length,
  };
}

/**
 * Append an occupancy sample when the site totals actually moved. Sampling on
 * change rather than on a timer keeps the chart a faithful step function of the
 * session and keeps the buffer proportional to real activity, not to uptime.
 */
function withTimeline(state: LiveState, now: number): LiveState {
  const totals = { free: 0, occupied: 0, error: 0 };

  for (const entry of Object.values(state.entries)) {
    if (!entry.state) continue;

    const counts = countSlots(entry.state.slots);

    totals.free += counts.free;
    totals.occupied += counts.occupied;
    totals.error += counts.error;
  }

  const last = state.timeline.at(-1);

  if (
    last &&
    last.free === totals.free &&
    last.occupied === totals.occupied &&
    last.error === totals.error
  ) {
    return state;
  }

  return {
    ...state,
    timeline: [...state.timeline, { t: now, ...totals }].slice(-TIMELINE_LIMIT),
  };
}
