/**
 * Root page: server component that pre-fetches the first snapshot.
 *
 * Doing the first fetch on the server means the page arrives with real
 * availability data already rendered -- no loading skeleton on first paint and
 * no hydration mismatch because the client seeds its store with the same
 * sections the server rendered.
 *
 * The failure path is explicit: if the backend is unreachable at request time
 * we still render the page, but with no sections and an error notice in the
 * activity feed. The WebSocket and REST fallback then take over in the browser.
 */
import { TooltipProvider } from "@/components/ui/tooltip";
import { listSections } from "@/lib/api/parking";
import { LiveProvider } from "@/lib/live/provider";
import { DashboardHeader } from "@/components/shell/header";
import { StatusBar } from "@/components/shell/status-bar";
import { DashboardBoard } from "@/components/parking/dashboard-board";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Smart Parking — Dashboard",
};

// Never cache: availability is real-time data.
export const dynamic = "force-dynamic";

export default async function HomePage() {
  let initialSections = null;
  let initialError: string | null = null;

  try {
    initialSections = await listSections();
  } catch (err) {
    initialError =
      err instanceof Error ? err.message : "backend unavailable at page load";
  }

  return (
    <TooltipProvider delayDuration={300}>
      <LiveProvider
        initialSections={initialSections ?? []}
        initialError={initialError}
      >
        <div className="flex min-h-screen flex-col">
          <DashboardHeader />

          <main className="flex-1 p-4 lg:p-6">
            <DashboardBoard />
          </main>

          <StatusBar />
        </div>
      </LiveProvider>
    </TooltipProvider>
  );
}
