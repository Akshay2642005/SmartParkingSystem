import { type VariantProps, cva } from "class-variance-authority";

import { cn } from "@/lib/utils";
import type { SlotState } from "@/lib/parking/contract";
import type { SectionHealth } from "@/lib/parking/derive";

/*
 * Colour is reserved for state. Only four things are allowed to be saturated on
 * screen: free (green), occupied (slate), fault (amber), offline/stale (red).
 * Any other colour would dilute the signal.
 */

export const slotStateVariants = cva(
  "inline-flex items-center justify-center font-mono text-[0.625rem] font-bold leading-none tabular-nums uppercase tracking-wide transition-colors",
  {
    variants: {
      state: {
        free: "text-free",
        occupied: "text-occupied",
        error: "text-fault",
      } satisfies Record<SlotState, string>,
    },
  },
);

/**
 * The little coloured square that fills one cell on the bay grid.
 * Click-target size comes from the parent cell, not this element.
 */
export function SlotCell({
  state,
  id,
  className,
}: {
  state: SlotState;
  id: string;
  className?: string;
}) {
  return (
    <div
      title={`${id}: ${state}`}
      aria-label={`slot ${id} ${state}`}
      className={cn(
        "pixel-frame flex aspect-square w-full items-center justify-center transition-colors duration-300",
        state === "free" && "bg-free-soft text-free",
        state === "occupied" && "bg-occupied-soft text-occupied",
        state === "error" && "bg-fault-soft text-fault",
        className,
      )}
    >
      <span className="text-label" aria-hidden>
        {state === "free" ? "F" : state === "occupied" ? "O" : "!"}
      </span>
    </div>
  );
}

/**
 * Node / section health badge. One text token, no icon, no border radius —
 * status chips on a console should be read at a glance, not decoded.
 */
export const healthVariants = cva(
  "text-label inline-flex items-center gap-1 rounded-sm px-1.5 py-0.5",
  {
    variants: {
      health: {
        live: "bg-free-soft text-free",
        stale: "bg-fault-soft text-fault",
        offline: "bg-offline-soft text-offline",
        unreported: "bg-muted text-muted-foreground",
      } satisfies Record<SectionHealth, string>,
    },
  },
);

export function HealthBadge({
  health,
  className,
}: {
  health: SectionHealth;
  className?: string;
} & VariantProps<typeof healthVariants>) {
  const label =
    health === "live"
      ? "live"
      : health === "stale"
        ? "stale"
        : health === "offline"
          ? "offline"
          : "no data";

  return (
    <span className={cn(healthVariants({ health }), className)}>
      <span
        aria-hidden
        className={cn(
          "inline-block size-1.5 rounded-full",
          health === "live" && "bg-free",
          health === "stale" && "bg-fault",
          health === "offline" && "bg-offline",
          health === "unreported" && "bg-muted-foreground",
        )}
      />
      {label}
    </span>
  );
}
