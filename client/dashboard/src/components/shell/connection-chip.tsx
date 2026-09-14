"use client";

import { cn } from "@/lib/utils";
import { useConnection } from "@/lib/live/hooks";
import type { ConnectionPhase, LiveSource } from "@/lib/live/reducer";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/**
 * Connection status chip in the header.
 *
 * Three states visible to the operator:
 *
 * - Solid green dot · **live** — socket open, data flowing.
 * - Pulsing amber dot · **reconnecting** — socket dropped; REST polling active.
 * - Static red dot · **degraded** — socket stopped; REST fallback running.
 *
 * The label says which transport is carrying data (`socket` vs `rest`) so an
 * operator knows whether they are seeing live push or a polled snapshot.
 */
export function ConnectionChip({ className }: { className?: string }) {
  const connection = useConnection();

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div
          className={cn(
            "text-label inline-flex cursor-default items-center gap-1.5 rounded-sm px-2 py-1",
            chipBg(connection.phase, connection.source),
            className,
          )}
          aria-label={`Connection: ${connectionLabel(connection.phase, connection.source)}`}
        >
          <Dot phase={connection.phase} />
          <span className={chipText(connection.phase, connection.source)}>
            {connectionLabel(connection.phase, connection.source)}
          </span>
        </div>
      </TooltipTrigger>

      <TooltipContent side="bottom" align="end" className="font-mono text-xs max-w-60">
        <ChipDetail phase={connection.phase} source={connection.source} error={connection.lastError} />
      </TooltipContent>
    </Tooltip>
  );
}

function Dot({ phase }: { phase: ConnectionPhase }) {
  return (
    <span
      aria-hidden
      className={cn(
        "inline-block size-1.5 rounded-full",
        phase === "open" && "bg-free",
        (phase === "connecting" || phase === "reconnecting") &&
          "bg-fault animate-pulse",
        (phase === "stopped" || phase === "idle") && "bg-offline",
      )}
    />
  );
}

function connectionLabel(phase: ConnectionPhase, source: LiveSource): string {
  switch (phase) {
    case "open":
      return source === "rest" ? "rest" : "live";
    case "connecting":
      return "connecting";
    case "reconnecting":
      return "reconnecting";
    case "stopped":
      return "stopped";
    case "idle":
      return "idle";
  }
}

function chipBg(phase: ConnectionPhase, _source: LiveSource): string {
  if (phase === "open") return "bg-free-soft";
  if (phase === "connecting" || phase === "reconnecting") return "bg-fault-soft";

  return "bg-offline-soft";
}

function chipText(phase: ConnectionPhase, _source: LiveSource): string {
  if (phase === "open") return "text-free";
  if (phase === "connecting" || phase === "reconnecting") return "text-fault";

  return "text-offline";
}

function ChipDetail({
  phase,
  source,
  error,
}: {
  phase: ConnectionPhase;
  source: LiveSource;
  error: string | null;
}) {
  return (
    <div className="space-y-1">
      <div className="flex gap-3">
        <span className="text-muted-foreground w-14">phase</span>
        <span>{phase}</span>
      </div>
      <div className="flex gap-3">
        <span className="text-muted-foreground w-14">transport</span>
        <span>{source === "none" ? "—" : source}</span>
      </div>
      {error && (
        <div className="text-fault pt-1 border-t border-border">{error}</div>
      )}
    </div>
  );
}
