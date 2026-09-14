"use client";

import { cn } from "@/lib/utils";
import { formatAge, formatPercent, formatTimestampUtc } from "@/lib/parking/format";
import { compareSlots } from "@/lib/parking/derive";
import type { SectionView } from "@/lib/parking/derive";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { HealthBadge, SlotCell } from "./slot-state-badge";
import { useNow } from "@/lib/live/use-now";

/** One section's card on the availability board. */
export function SectionCard({
  view,
  className,
}: {
  view: SectionView;
  className?: string;
}) {
  const now = useNow();
  const ageMs = view.state
    ? Math.max(0, (now || Date.now()) - view.state.server_ts_ms)
    : null;

  const freeCount = view.totals.free;
  const totalKnown = view.totals.free + view.totals.occupied;

  const sortedSlots = view.state
    ? [...view.state.slots].sort(compareSlots)
    : [];

  return (
    <Card
      className={cn(
        "relative transition-shadow hover:shadow-md",
        view.health === "offline" && "opacity-60",
        className,
      )}
    >
      {/* Accent strip: the one pixel of colour that tells you the section
          state at a glance from across the room. */}
      <div
        aria-hidden
        className={cn(
          "absolute inset-x-0 top-0 h-0.5 rounded-t-xl",
          view.health === "live" && "bg-free",
          view.health === "stale" && "bg-fault",
          view.health === "offline" && "bg-offline",
          view.health === "unreported" && "bg-border",
        )}
      />

      <CardHeader className="gap-2 pb-3">
        <div className="flex items-start justify-between gap-2">
          <div>
            <CardTitle className="font-mono text-lg font-bold tracking-tight">
              Section {view.section}
            </CardTitle>
            <CardDescription className="text-label mt-0.5">
              {view.site}
            </CardDescription>
          </div>
          <HealthBadge health={view.health} />
        </div>

        {/* Headline number: free slots. */}
        {view.health !== "offline" && view.health !== "unreported" ? (
          <div className="flex items-baseline gap-2">
            <span className="text-metric">{freeCount}</span>
            <span className="text-muted-foreground text-sm">
              / {totalKnown} free
            </span>
            {view.totals.error > 0 && (
              <span className="text-label ml-auto text-fault">
                {view.totals.error} fault{view.totals.error > 1 ? "s" : ""}
              </span>
            )}
          </div>
        ) : (
          <div className="text-muted-foreground text-sm">
            {view.health === "offline" ? "Node offline" : "Awaiting first report"}
          </div>
        )}
      </CardHeader>

      <CardContent>
        {/* Bay grid — pixel-frame cells. The 8-bit motif is used here only,
            so the retro language marks "this is a physical bay". */}
        {sortedSlots.length > 0 ? (
          <div
            className="grid gap-1"
            style={{
              gridTemplateColumns: `repeat(${Math.min(sortedSlots.length, 4)}, minmax(0, 1fr))`,
            }}
          >
            {sortedSlots.map((slot) => (
              <Tooltip key={slot.id}>
                <TooltipTrigger asChild>
                  <button
                    type="button"
                    aria-label={`${slot.id}: ${slot.state}`}
                    className="focus-visible:outline-ring/50 focus-visible:outline-2 focus-visible:outline-offset-1 rounded-sm"
                  >
                    <SlotCell state={slot.state} id={slot.id} />
                  </button>
                </TooltipTrigger>
                <TooltipContent side="top" className="font-mono text-xs">
                  <span className="font-bold">{slot.id}</span>
                  <span className="text-muted-foreground ml-2">{slot.state}</span>
                </TooltipContent>
              </Tooltip>
            ))}
          </div>
        ) : (
          <div
            className="grid gap-1"
            style={{
              gridTemplateColumns: `repeat(${Math.min(
                view.state?.slot_count ?? 3,
                4,
              )}, minmax(0, 1fr))`,
            }}
          >
            {Array.from({ length: view.state?.slot_count ?? 3 }).map((_, i) => (
              // biome-ignore lint/suspicious/noArrayIndexKey: static skeleton
              <Skeleton key={i} className="aspect-square w-full" />
            ))}
          </div>
        )}

        {/* Occupancy bar */}
        {view.occupancy !== null && (
          <div className="mt-3 flex items-center gap-2">
            <div className="bg-muted h-1 flex-1 overflow-hidden rounded-full">
              <div
                className="h-full rounded-full bg-occupied transition-all duration-500"
                style={{ width: `${Math.round(view.occupancy * 100)}%` }}
              />
            </div>
            <span className="text-label text-muted-foreground w-8 text-right">
              {formatPercent(view.occupancy)}
            </span>
          </div>
        )}

        {/* Timestamp */}
        {ageMs !== null && (
          <div className="mt-2 flex justify-between">
            <Tooltip>
              <TooltipTrigger asChild>
                <span className="text-label text-muted-foreground cursor-default">
                  {ageMs === 0 ? "just now" : `${formatAge(ageMs)} ago`}
                </span>
              </TooltipTrigger>
              <TooltipContent>
                {view.state
                  ? formatTimestampUtc(view.state.server_ts_ms)
                  : "—"}
              </TooltipContent>
            </Tooltip>
            {view.state && (
              <span className="text-label text-muted-foreground">
                seq {view.state.seq}
              </span>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export function SectionCardSkeleton() {
  return (
    <Card>
      <CardHeader className="gap-2 pb-3">
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <Skeleton className="h-5 w-24" />
            <Skeleton className="h-3 w-12" />
          </div>
          <Skeleton className="h-4 w-12 rounded-sm" />
        </div>
        <Skeleton className="h-8 w-16" />
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-3 gap-1">
          {[0, 1, 2].map((i) => (
            <Skeleton key={i} className="aspect-square w-full" />
          ))}
        </div>
        <Skeleton className="mt-3 h-1 w-full" />
        <Skeleton className="mt-2 h-3 w-20" />
      </CardContent>
    </Card>
  );
}
