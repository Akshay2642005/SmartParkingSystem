"use client";

/**
 * What components read.
 *
 * Each hook selects only stable references out of the store — the `entries`
 * map, the activity array — and derives everything else with `useMemo`. That
 * split matters twice over: Zustand requires selectors to return a stable
 * snapshot (a fresh array on every render would loop), and it keeps every rule
 * in the pure functions of `parking/derive.ts` where it can be tested without a
 * renderer.
 *
 * Views are recomputed once per clock tick because staleness is a function of
 * time, not of any store change. That is one `useMemo` per second over a
 * handful of sections — far cheaper than the alternative of writing the clock
 * into the store and re-notifying every subscriber.
 */
import { useMemo } from "react";
import { useStore } from "zustand";

import { sectionKey } from "@/lib/parking/contract";
import {
  type SectionView,
  type SiteTotals,
  compareSections,
  sitesOf,
  summariseSite,
  toSectionView,
} from "@/lib/parking/derive";
import { useLiveStoreApi } from "./provider";
import type {
  ActivityEntry,
  ActivityKind,
  ConnectionState,
  LiveSource,
  OccupancySample,
} from "./reducer";
import { useNow } from "./use-now";

/** Every known section in board order, with staleness resolved against now. */
export function useSectionViews(): SectionView[] {
  const store = useLiveStoreApi();
  const entries = useStore(store, (state) => state.entries);
  const now = useNow();

  return useMemo(() => {
    const clock = now || Date.now();

    return Object.entries(entries)
      .map(([key, entry]) =>
        toSectionView(key, entry.state, entry.nodeStatus, clock),
      )
      .sort(compareSections);
  }, [entries, now]);
}

export function useSectionView(
  site: string,
  section: string,
): SectionView | null {
  const store = useLiveStoreApi();
  const key = sectionKey(site, section);
  const entry = useStore(store, (state) => state.entries[key]);
  const now = useNow();

  return useMemo(
    () =>
      entry
        ? toSectionView(key, entry.state, entry.nodeStatus, now || Date.now(), {
            site,
            section,
          })
        : null,
    [entry, key, now, site, section],
  );
}

export function useSiteTotals(views: readonly SectionView[]): SiteTotals {
  return useMemo(() => summariseSite(views), [views]);
}

export function useSites(views: readonly SectionView[]): string[] {
  return useMemo(() => sitesOf(views), [views]);
}

export interface LiveConnection extends ConnectionState {
  lastFrameAt: number | null;
  lastSnapshotAt: number | null;
  source: LiveSource;
  /** Age of the last frame in ms, or `null` before the clock is available. */
  silenceMs: number | null;
}

export function useConnection(): LiveConnection {
  const store = useLiveStoreApi();
  const connection = useStore(store, (state) => state.connection);
  const lastFrameAt = useStore(store, (state) => state.lastFrameAt);
  const lastSnapshotAt = useStore(store, (state) => state.lastSnapshotAt);
  const source = useStore(store, (state) => state.source);
  const now = useNow();

  return useMemo(
    () => ({
      ...connection,
      lastFrameAt,
      lastSnapshotAt,
      source,
      silenceMs: now && lastFrameAt ? Math.max(0, now - lastFrameAt) : null,
    }),
    [connection, lastFrameAt, lastSnapshotAt, source, now],
  );
}

export function useActivity(options?: {
  kinds?: readonly ActivityKind[];
  site?: string;
  section?: string;
  limit?: number;
}): ActivityEntry[] {
  const store = useLiveStoreApi();
  const activity = useStore(store, (state) => state.activity);
  const kinds = options?.kinds;
  const site = options?.site;
  const section = options?.section;
  const limit = options?.limit;

  return useMemo(() => {
    let filtered = activity;

    if (kinds && kinds.length > 0) {
      filtered = filtered.filter((entry) => kinds.includes(entry.kind));
    }

    if (site !== undefined || section !== undefined) {
      filtered = filtered.filter((entry) => {
        if (!("site" in entry)) return false;

        return (
          (site === undefined || entry.site === site) &&
          (section === undefined || entry.section === section)
        );
      });
    }

    return limit === undefined ? filtered : filtered.slice(0, limit);
  }, [activity, kinds, site, section, limit]);
}

export function useTimeline(): OccupancySample[] {
  const store = useLiveStoreApi();

  return useStore(store, (state) => state.timeline);
}
