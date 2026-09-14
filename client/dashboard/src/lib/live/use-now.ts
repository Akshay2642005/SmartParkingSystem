"use client";

/**
 * One clock for the whole console.
 *
 * A live dashboard renders many ages at once ("12s ago", "stale for 2m"), and
 * giving each of them its own interval would mean dozens of timers waking the
 * tab out of step. This is a single 1 s ticker behind `useSyncExternalStore`,
 * so every age on screen advances in the same frame.
 *
 * It returns **0 until the component has mounted**, on the server and through
 * the hydration render alike. That is the contract that keeps time out of
 * hydration: server HTML and the first client render agree by construction, and
 * callers treat 0 as "clock not available yet" and render a stable absolute
 * value (`formatTimestampUtc`) instead of an age.
 */
import { useEffect, useState, useSyncExternalStore } from "react";

const TICK_MS = 1_000;

const listeners = new Set<() => void>();
let current = 0;
let timer: ReturnType<typeof setInterval> | null = null;

function subscribe(listener: () => void): () => void {
  listeners.add(listener);

  if (timer === null) {
    current = Date.now();
    timer = setInterval(() => {
      current = Date.now();

      for (const notify of listeners) notify();
    }, TICK_MS);
  }

  return () => {
    listeners.delete(listener);

    if (listeners.size === 0 && timer !== null) {
      clearInterval(timer);
      timer = null;
    }
  };
}

function getSnapshot(): number {
  if (current === 0) current = Date.now();

  return current;
}

function getServerSnapshot(): number {
  return 0;
}

/** Current wall clock in ms, or 0 before mount. */
export function useNow(): number {
  const tick = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot);
  const [mounted, setMounted] = useState(false);

  useEffect(() => setMounted(true), []);

  return mounted ? tick : 0;
}
