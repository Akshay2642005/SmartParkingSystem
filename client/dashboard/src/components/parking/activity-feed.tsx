"use client";

import { cn } from "@/lib/utils";
import { formatAgo, formatTimestampUtc } from "@/lib/parking/format";
import { useActivity } from "@/lib/live/hooks";
import { useNow } from "@/lib/live/use-now";
import type { ActivityEntry, ActivityKind } from "@/lib/live/reducer";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/**
 * Session event log.
 *
 * The feed is read newest-first: operators glance at the top, not the bottom.
 * Each entry has a kind badge, a brief description, and a relative timestamp
 * that updates with the clock. Hovering shows the exact UTC timestamp.
 *
 * Slot transitions carry the full from→to labels because "A-1: free→occupied"
 * is unambiguous and requires no colour lookup. Connection events use
 * un-accented text so they do not compete with the slot state palette. Error
 * events get the fault colour because they are the operator's action item.
 */
export function ActivityFeed({
  site,
  section,
  kinds,
  limit = 60,
  className,
}: {
  site?: string;
  section?: string;
  kinds?: readonly ActivityKind[];
  limit?: number;
  className?: string;
}) {
  const entries = useActivity({ site, section, kinds, limit });
  const now = useNow();

  if (entries.length === 0) {
    return (
      <div
        className={cn(
          "flex items-center justify-center text-muted-foreground text-sm h-40",
          className,
        )}
      >
        No events yet
      </div>
    );
  }

  return (
    <ScrollArea className={cn("h-64 pr-1", className)}>
      <ol className="space-y-px" aria-label="Activity feed">
        {entries.map((entry) => (
          <ActivityRow key={entry.id} entry={entry} now={now} />
        ))}
      </ol>
    </ScrollArea>
  );
}

function ActivityRow({
  entry,
  now,
}: {
  entry: ActivityEntry;
  now: number;
}) {
  const clock = now || Date.now();
  const ageMs = Math.max(0, clock - entry.at);

  return (
    <li className="flex items-start gap-2 py-1 font-mono text-xs">
      <KindBadge entry={entry} />
      <span className="flex-1 text-foreground/80 leading-snug">
        <EntryLabel entry={entry} />
      </span>
      <Tooltip>
        <TooltipTrigger asChild>
          <span className="shrink-0 text-muted-foreground tabular-nums cursor-default">
            {ageMs < 1000 ? "now" : formatAgo(ageMs)}
          </span>
        </TooltipTrigger>
        <TooltipContent side="left">
          {formatTimestampUtc(entry.at)}
        </TooltipContent>
      </Tooltip>
    </li>
  );
}

function KindBadge({ entry }: { entry: ActivityEntry }) {
  const kindLabel =
    entry.kind === "slot"
      ? "slot"
      : entry.kind === "node"
        ? "node"
        : entry.kind === "snapshot"
          ? "sync"
          : entry.kind === "connection"
            ? "conn"
            : "err";

  return (
    <span
      className={cn(
        "text-label shrink-0 rounded-sm px-1 py-0.5",
        entry.kind === "slot" && "bg-occupied-soft text-occupied",
        entry.kind === "node" && "bg-muted text-muted-foreground",
        entry.kind === "snapshot" && "bg-free-soft text-free",
        entry.kind === "connection" && "bg-muted text-muted-foreground",
        entry.kind === "error" && "bg-fault-soft text-fault",
      )}
    >
      {kindLabel}
    </span>
  );
}

function EntryLabel({ entry }: { entry: ActivityEntry }) {
  switch (entry.kind) {
    case "slot":
      return (
        <>
          <span className="font-bold">{entry.slotId}</span>
          <span className="text-muted-foreground"> {entry.section} · </span>
          <StateToken state={entry.from} /> → <StateToken state={entry.to} />
        </>
      );
    case "node":
      return (
        <>
          <span className="font-bold">
            {entry.site}/{entry.section}
          </span>{" "}
          <span
            className={cn(
              entry.status === "online" ? "text-free" : "text-offline",
            )}
          >
            {entry.status}
          </span>
        </>
      );
    case "snapshot":
      return (
        <span className="text-muted-foreground">
          snapshot · {entry.sections} section
          {entry.sections !== 1 ? "s" : ""} via {entry.source}
        </span>
      );
    case "connection":
      return (
        <span className="text-muted-foreground">
          {entry.phase}
          {entry.detail ? ` · ${entry.detail}` : ""}
        </span>
      );
    case "error":
      return (
        <span className="text-fault">
          [{entry.code}] {entry.message}
        </span>
      );
  }
}

function StateToken({ state }: { state: string }) {
  return (
    <span
      className={cn(
        state === "free" && "text-free",
        state === "occupied" && "text-occupied",
        state === "error" && "text-fault",
      )}
    >
      {state}
    </span>
  );
}
