"use client";

/**
 * Client-side context tree.
 *
 * Three providers, in dependency order:
 *
 * 1. `ThemeProvider` — sets the `dark` class; must wrap everything that reads
 *    theme tokens.
 * 2. `QueryClientProvider` — gives React Query access to the cache; must wrap
 *    `LiveProvider` which uses `useQuery` for the REST fallback.
 * 3. `LiveProvider` — the parking state store and socket, seeded with nothing
 *    on this boundary; each page passes `initialSections` directly to avoid
 *    prop-drilling through a context that isn’t needed during the theme flash.
 *
 * `ThemeProvider` alone lives here so the dark class is applied even before
 * the page JavaScript has run (it reads `localStorage` synchronously).
 * Everything else is deferred to the page’s own `LiveProvider` so the initial
 * sections can be passed in at that level.
 */
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "@teispace/next-themes";
import type { ReactNode } from "react";
import { useState } from "react";

export function Providers({ children }: { children: ReactNode }) {
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            // Availability is real-time; the socket pushes updates. Polling via
            // React Query is the fallback, not the primary path, and stale data
            // is worse than no data on an availability board.
            staleTime: 0,
            gcTime: 60_000,
            retry: 1,
            refetchOnWindowFocus: false,
          },
        },
      }),
  );

  return (
    <ThemeProvider
      attribute="class"
      defaultTheme="system"
      enableSystem
      disableTransitionOnChange
    >
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    </ThemeProvider>
  );
}
