"use client";

import { Moon, Sun, ParkingSquare } from "lucide-react";
import { useTheme } from "@teispace/next-themes";
import { Button } from "@/components/ui/button";
import { ConnectionChip } from "./connection-chip";
import { cn } from "@/lib/utils";

export function DashboardHeader({ className }: { className?: string }) {
  const { resolvedTheme, setTheme } = useTheme();

  return (
    <header
      className={cn(
        "sticky top-0 z-40 flex h-12 shrink-0 items-center gap-3 border-b bg-background/90 px-4 backdrop-blur-sm",
        className,
      )}
    >
      {/* Wordmark */}
      <div className="flex items-center gap-2 text-foreground">
        <ParkingSquare className="size-4 shrink-0 text-primary" strokeWidth={2.5} />
        <span className="font-mono text-sm font-bold tracking-tight">
          Smart Parking
        </span>
        <span
          className="text-label rounded bg-muted px-1 py-0.5 text-muted-foreground"
          title="API version 1"
        >
          v1
        </span>
      </div>

      <div className="flex-1" />

      <ConnectionChip />

      <Button
        variant="ghost"
        size="icon"
        aria-label={resolvedTheme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
        onClick={() =>
          setTheme(resolvedTheme === "dark" ? "light" : "dark")
        }
      >
        {resolvedTheme === "dark" ? (
          <Sun className="size-4" />
        ) : (
          <Moon className="size-4" />
        )}
      </Button>
    </header>
  );
}
