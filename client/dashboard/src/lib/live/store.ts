/**
 * The live store: `reducer.ts` wrapped in Zustand.
 *
 * Created **per provider instance**, not as a module singleton. On the server a
 * module-level store would be shared by every concurrent request, and seeding
 * it with one request's data would leak into another's. A store built inside
 * the provider is also what makes server-side rendering of live data possible
 * at all: the same initial sections produce the same first render on both
 * sides, so hydration matches.
 *
 * The store holds no derivations. Everything the UI reads is computed from
 * `entries` by the pure functions in `parking/derive.ts`, called from the hooks
 * in `hooks.ts`, so the store never has to be invalidated when a rule changes.
 */
import { createStore } from "zustand/vanilla";

import type { SectionState, ServerEvent } from "@/lib/parking/contract";
import {
  type ConnectionPhase,
  type LiveState,
  applyConnectionPhase,
  applyProtocolError,
  applyRestSnapshot,
  applyServerEvent,
  initialLiveState,
} from "./reducer";

export interface LiveActions {
  /** A validated frame from the WebSocket. */
  ingest(event: ServerEvent, now?: number): void;
  /** A section list from REST: server-rendered first paint, or the fallback poll. */
  ingestSections(sections: readonly SectionState[], now?: number): void;
  setPhase(
    phase: ConnectionPhase,
    info?: { attempt?: number; detail?: string },
    now?: number,
  ): void;
  /** A frame that did not match the contract — a version-skew signal. */
  reportProtocolError(message: string, now?: number): void;
}

export type LiveStore = LiveState & LiveActions;

export type LiveStoreApi = ReturnType<typeof createLiveStore>;

export function createLiveStore(
  initialSections: readonly SectionState[] = [],
  now: number = Date.now(),
) {
  const seeded =
    initialSections.length > 0
      ? applyRestSnapshot(initialLiveState(), initialSections, now)
      : initialLiveState();

  return createStore<LiveStore>()((set) => ({
    ...seeded,

    ingest: (event, at = Date.now()) =>
      set((state) => applyServerEvent(state, event, at)),

    ingestSections: (sections, at = Date.now()) =>
      set((state) => applyRestSnapshot(state, sections, at)),

    setPhase: (phase, info = {}, at = Date.now()) =>
      set((state) => applyConnectionPhase(state, phase, at, info)),

    reportProtocolError: (message, at = Date.now()) =>
      set((state) => applyProtocolError(state, message, at)),
  }));
}
