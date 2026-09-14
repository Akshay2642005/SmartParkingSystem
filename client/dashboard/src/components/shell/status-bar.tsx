"use client";

import { useQuery } from "@tanstack/react-query";
import { getStatus } from "@/lib/api/parking";
import { formatUptime, formatTimestampUtc } from "@/lib/parking/format";
import { cn } from "@/lib/utils";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

/**
 * Thin status bar at the bottom of the page.
 * Polls `GET /status` every 30 s; shows service health, uptime, and subscriber
 * count. These are deliberately quiet — the board's colour is the signal, not
 * this bar.
 */
export function StatusBar({ className }: { className?: string }) {
  const { data, error } = useQuery({
    queryKey: ["status"],
    queryFn: ({ signal }) => getStatus({ signal }),
    refetchInterval: 30_000,
    staleTime: 0,
  });

  return (
    <footer
      className={cn(
        "border-t bg-background/80 px-4 py-1.5 text-muted-foreground",
        className,
      )}
    >
      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-[0.6875rem]">
        {data ? (
          <>
            {/* Service health */}
            <StatusItem
              label="svc"
              value={data.status}
              colourClass={
                data.status === "healthy" ? "text-free" : "text-fault"
              }
            />

            {/* MQTT ingest */}
            <StatusItem
              label="ingest"
              value={data.checks.ingest.status}
              colourClass={
                data.checks.ingest.status === "healthy"
                  ? "text-free"
                  : "text-fault"
              }
              detail={data.checks.ingest.detail}
            />

            {/* Store */}
            <StatusItem
              label="store"
              value={data.checks.store.status}
              colourClass={
                data.checks.store.status === "healthy"
                  ? "text-free"
                  : "text-fault"
              }
              detail={data.checks.store.detail}
            />

            <span className="text-border">|</span>

            <StatusItem
              label="sections"
              value={String(data.sections)}
            />
            <StatusItem
              label="clients"
              value={String(data.subscribers)}
            />
            <StatusItem
              label="uptime"
              value={formatUptime(data.uptime_secs)}
            />

            <span className="text-border">|</span>

            <Tooltip>
              <TooltipTrigger asChild>
                <span className="cursor-default">
                  {data.service} {data.version} ({data.environment})
                </span>
              </TooltipTrigger>
              <TooltipContent side="top">
                {formatTimestampUtc(new Date(data.timestamp).getTime())}
              </TooltipContent>
            </Tooltip>
          </>
        ) : error ? (
          <span className="text-fault">/status unreachable</span>
        ) : (
          <span>loading…</span>
        )}
      </div>
    </footer>
  );
}

function StatusItem({
  label,
  value,
  colourClass,
  detail,
}: {
  label: string;
  value: string;
  colourClass?: string;
  detail?: string;
}) {
  const item = (
    <span className="inline-flex items-baseline gap-1">
      <span className="text-muted-foreground/60">{label}</span>
      <span className={colourClass}>{value}</span>
    </span>
  );

  if (!detail) return item;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="cursor-default">{item}</span>
      </TooltipTrigger>
      <TooltipContent side="top" className="font-mono text-xs">
        {detail}
      </TooltipContent>
    </Tooltip>
  );
}
