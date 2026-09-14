"use client";

import {
  Area,
  AreaChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { Payload } from "recharts/types/component/DefaultTooltipContent";

import { cn } from "@/lib/utils";
import { formatAge, formatTimestampUtc } from "@/lib/parking/format";
import type { OccupancySample } from "@/lib/live/reducer";
import { useNow } from "@/lib/live/use-now";
import { useTimeline } from "@/lib/live/hooks";

/**
 * Session occupancy history as a stacked area chart.
 *
 * The chart is session-local: no persistence exists (`database.md`), so this is
 * all the history there is. The timeline is capped at 240 samples which is 4 h
 * at one sample per minute of real activity — plenty for a shift overview.
 *
 * The three areas (free / occupied / error) are stacked so their sum always
 * equals the total slot count. A shrinking free band means bays are filling;
 * a growing fault band tells the operator something is wrong independently of
 * whether anyone is parked.
 *
 * recharts is used directly here rather than through the 8bitcn chart wrapper
 * because this chart needs a custom tooltip and the 8bit wrapper's API surface
 * is optimised for single-series bar charts from the block library. The design
 * tokens (`--color-free`, etc.) still drive the colours so the chart is
 * coherent with the rest of the board.
 */
export function OccupancyChart({ className }: { className?: string }) {
  const timeline = useTimeline();
  const now = useNow();

  if (timeline.length < 2) {
    return (
      <div
        className={cn(
          "flex items-center justify-center rounded-lg border border-dashed text-muted-foreground text-sm h-40",
          className,
        )}
      >
        Collecting data…
      </div>
    );
  }

  const clock = now || Date.now();

  // Use age-ago labels on the X axis, pinned to the latest sample.
  const latest = timeline[timeline.length - 1]?.t ?? clock;

  return (
    <div className={cn("h-40 w-full", className)}>
      <ResponsiveContainer width="100%" height="100%">
        <AreaChart
          data={timeline}
          margin={{ top: 4, right: 0, left: -24, bottom: 0 }}
        >
          <defs>
            <linearGradient id="gradFree" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="5%"
                stopColor="var(--color-free)"
                stopOpacity={0.3}
              />
              <stop
                offset="95%"
                stopColor="var(--color-free)"
                stopOpacity={0.05}
              />
            </linearGradient>
            <linearGradient id="gradOccupied" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="5%"
                stopColor="var(--color-occupied)"
                stopOpacity={0.5}
              />
              <stop
                offset="95%"
                stopColor="var(--color-occupied)"
                stopOpacity={0.1}
              />
            </linearGradient>
            <linearGradient id="gradFault" x1="0" y1="0" x2="0" y2="1">
              <stop
                offset="5%"
                stopColor="var(--color-fault)"
                stopOpacity={0.6}
              />
              <stop
                offset="95%"
                stopColor="var(--color-fault)"
                stopOpacity={0.1}
              />
            </linearGradient>
          </defs>

          <CartesianGrid
            strokeDasharray="3 3"
            stroke="var(--color-border)"
            vertical={false}
          />

          <XAxis
            dataKey="t"
            tickLine={false}
            axisLine={false}
            tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
            tickFormatter={(t: number) => {
              const ageMs = Math.max(0, latest - t);

              return ageMs < 2000 ? "now" : `-${formatAge(ageMs)}`;
            }}
            interval="preserveStartEnd"
          />

          <YAxis
            tickLine={false}
            axisLine={false}
            allowDecimals={false}
            tick={{ fontSize: 10, fill: "var(--color-muted-foreground)" }}
          />

          <Tooltip
            content={<ChartTooltip />}
            cursor={{ stroke: "var(--color-border)", strokeWidth: 1 }}
          />

          {/* Stack order: fault on top so it is visible even when bays are full */}
          <Area
            type="step"
            dataKey="free"
            stackId="1"
            stroke="var(--color-free)"
            strokeWidth={1.5}
            fill="url(#gradFree)"
            isAnimationActive={false}
          />
          <Area
            type="step"
            dataKey="occupied"
            stackId="1"
            stroke="var(--color-occupied)"
            strokeWidth={1.5}
            fill="url(#gradOccupied)"
            isAnimationActive={false}
          />
          <Area
            type="step"
            dataKey="error"
            stackId="1"
            stroke="var(--color-fault)"
            strokeWidth={1.5}
            fill="url(#gradFault)"
            isAnimationActive={false}
          />
        </AreaChart>
      </ResponsiveContainer>
    </div>
  );
}

interface ChartTooltipProps {
  active?: boolean;
  payload?: Payload<number, string>[];
  label?: number;
}

function ChartTooltip({ active, payload, label }: ChartTooltipProps) {
  if (!active || !payload?.length) return null;

  const find = (key: string) =>
    payload.find((p: Payload<number, string>) => p.dataKey === key)?.value ?? 0;
  const free = find("free");
  const occupied = find("occupied");
  const error = find("error");

  return (
    <div className="bg-popover text-popover-foreground rounded-md border px-3 py-2 shadow-lg font-mono text-xs space-y-1">
      <div className="text-muted-foreground mb-1">
        {formatTimestampUtc(label as number)}
      </div>
      <Row colour="var(--color-free)" label="free" value={free} />
      <Row colour="var(--color-occupied)" label="occupied" value={occupied} />
      {(error as number) > 0 && (
        <Row colour="var(--color-fault)" label="fault" value={error} />
      )}
    </div>
  );
}

function Row({
  colour,
  label,
  value,
}: {
  colour: string;
  label: string;
  value: number;
}) {
  return (
    <div className="flex items-center gap-2">
      <span
        className="inline-block size-2 rounded-full"
        style={{ background: colour }}
      />
      <span className="text-muted-foreground w-16">{label}</span>
      <span className="font-bold tabular-nums">{value}</span>
    </div>
  );
}
