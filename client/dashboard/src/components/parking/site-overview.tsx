"use client";

import { cn } from "@/lib/utils";
import { formatPercent } from "@/lib/parking/format";
import type { SectionView, SiteTotals } from "@/lib/parking/derive";

/**
 * The summary row above the section grid: three big numbers and an occupancy
 * bar. Reads well from across a room; gives an operator the headline before
 * they look at individual bays.
 */
export function SiteOverview({
  site,
  totals,
  sections,
  className,
}: {
  site: string;
  totals: SiteTotals;
  sections: readonly SectionView[];
  className?: string;
}) {
  const occupancyPct =
    totals.occupancy !== null ? Math.round(totals.occupancy * 100) : null;

  return (
    <section aria-label={`${site} overview`} className={cn("space-y-4", className)}>
      <div className="flex items-baseline justify-between gap-4">
        <h2 className="font-mono text-sm font-bold uppercase tracking-widest text-muted-foreground">
          {site}
        </h2>
        <span className="text-label text-muted-foreground">
          {sections.length} section{sections.length !== 1 ? "s" : ""}
        </span>
      </div>

      {/* Three headline metrics */}
      <div className="grid grid-cols-3 gap-px rounded-lg overflow-hidden bg-border">
        <Metric
          label="free"
          value={totals.free}
          total={totals.total}
          colourClass="text-free"
        />
        <Metric
          label="occupied"
          value={totals.occupied}
          total={totals.total}
          colourClass="text-occupied"
        />
        <Metric
          label={totals.error > 0 ? "fault" : "error"}
          value={totals.error}
          total={totals.total}
          colourClass={totals.error > 0 ? "text-fault" : "text-muted-foreground"}
        />
      </div>

      {/* Stacked occupancy bar — three segments rather than a single ratio
          because a section at 100 % with 2 fault sensors is different from one
          at 100 % with 0. The bar shows the true composition. */}
      {totals.total > 0 && (
        <div>
          <div className="bg-muted flex h-2 overflow-hidden rounded-full">
            <div
              className="bg-occupied transition-all duration-500"
              style={{ width: `${(totals.occupied / totals.total) * 100}%` }}
            />
            <div
              className="bg-fault transition-all duration-500"
              style={{ width: `${(totals.error / totals.total) * 100}%` }}
            />
          </div>
          <div className="mt-1 flex justify-between">
            <span className="text-label text-muted-foreground">
              {occupancyPct !== null ? `${occupancyPct}% occupied` : "no data"}
            </span>
            <span className="text-label text-muted-foreground">
              {formatPercent(totals.free / totals.total)} free
            </span>
          </div>
        </div>
      )}
    </section>
  );
}

function Metric({
  label,
  value,
  total,
  colourClass,
}: {
  label: string;
  value: number;
  total: number;
  colourClass: string;
}) {
  return (
    <div className="bg-card flex flex-col items-center gap-0.5 px-3 py-4">
      <span className={cn("text-metric", colourClass)}>{value}</span>
      <span className="text-label text-muted-foreground">{label}</span>
      {total > 0 && (
        <span className="text-label text-muted-foreground/60">
          {formatPercent(value / total)}
        </span>
      )}
    </div>
  );
}
