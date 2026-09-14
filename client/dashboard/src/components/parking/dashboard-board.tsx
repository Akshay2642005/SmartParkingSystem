"use client";

import { cn } from "@/lib/utils";
import { useSectionViews, useSites, useSiteTotals } from "@/lib/live/hooks";
import { SectionCard, SectionCardSkeleton } from "./section-card";
import { SiteOverview } from "./site-overview";
import { OccupancyChart } from "./occupancy-chart";
import { ActivityFeed } from "./activity-feed";
import type { SectionView } from "@/lib/parking/derive";

/**
 * The full dashboard: site overview, section grid, chart, and activity feed.
 *
 * Layout is two-column on large screens: the section grid takes 2/3 and the
 * right panel holds the chart and feed. On smaller screens everything stacks.
 *
 * Data comes entirely from the live store. No props — the store is seeded by
 * the server in `page.tsx` and kept fresh by the WebSocket in `LiveProvider`.
 */
export function DashboardBoard({ className }: { className?: string }) {
  const views = useSectionViews();
  const sites = useSites(views);

  if (views.length === 0) {
    return <EmptyState />;
  }

  return (
    <div className={cn("space-y-8", className)}>
      {sites.map((site) => {
        const siteViews = views.filter((v) => v.site === site);
        return <SiteBoard key={site} site={site} views={siteViews} />;
      })}
    </div>
  );
}

function SiteBoard({
  site,
  views,
}: {
  site: string;
  views: SectionView[];
}) {
  const totals = useSiteTotals(views);

  return (
    <div className="space-y-6">
      <SiteOverview site={site} totals={totals} sections={views} />

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Section grid: 2/3 on large screens */}
        <div className="lg:col-span-2">
          <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
            {views.map((view) => (
              <SectionCard key={view.key} view={view} />
            ))}
          </div>
        </div>

        {/* Right panel: chart + activity */}
        <div className="space-y-6">
          <section aria-label="Occupancy history">
            <h3 className="text-label mb-3 text-muted-foreground">Session</h3>
            <OccupancyChart />
          </section>

          <section aria-label="Activity feed">
            <h3 className="text-label mb-3 text-muted-foreground">Activity</h3>
            <ActivityFeed site={site} />
          </section>
        </div>
      </div>
    </div>
  );
}

function EmptyState() {
  return (
    <div className="space-y-8">
      {/* Skeleton overview strip */}
      <div className="h-32 animate-pulse rounded-lg bg-muted" />

      <div className="grid gap-6 lg:grid-cols-3">
        <div className="grid gap-4 sm:grid-cols-2 lg:col-span-2">
          {[0, 1, 2].map((i) => (
            <SectionCardSkeleton key={i} />
          ))}
        </div>

        <div className="space-y-6">
          <div>
            <div className="text-label mb-3 text-muted-foreground">Session</div>
            <div className="flex h-40 items-center justify-center rounded-lg border border-dashed text-muted-foreground text-sm">
              Awaiting first snapshot…
            </div>
          </div>

          <div>
            <div className="text-label mb-3 text-muted-foreground">Activity</div>
            <ActivityFeed />
          </div>
        </div>
      </div>
    </div>
  );
}
