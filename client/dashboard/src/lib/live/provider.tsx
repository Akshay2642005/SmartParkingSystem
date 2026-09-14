"use client";

/**
 * Mounts the live store, the WebSocket, and the REST fallback.
 *
 * This is where the browser-only concerns live, and there are exactly three:
 *
 * 1. **One socket per document.** Opened on mount, closed on unmount. React
 *    Strict Mode mounts effects twice in development, which this survives
 *    because `LiveSocket.start()` is idempotent and `stop()` is total.
 * 2. **Reconnect on evidence, not on a timer.** `online` and
 *    `visibilitychange` are direct evidence that a previous failure no longer
 *    applies (network came back, laptop woke up), so they short-circuit the
 *    backoff instead of leaving a wall display stale for the remaining wait.
 * 3. **Degrade to polling.** If the socket cannot stay open — a proxy that
 *    strips upgrades is the usual cause — `GET {prefix}/sections` is polled
 *    instead. It returns the same `SectionState` shape, so the board keeps
 *    working with a slower refresh rather than going blank, and the header says
 *    which transport is in use.
 *
 * The provider renders its children immediately; it never gates the tree on a
 * connection. The server already fetched the first snapshot, so the console is
 * useful before the socket is up.
 */
import { useQuery } from "@tanstack/react-query";
import {
  createContext,
  type ReactNode,
  useContext,
  useEffect,
  useRef,
  useState,
} from "react";

import { listSections } from "@/lib/api/parking";
import { websocketUrl } from "@/lib/api/endpoints";
import type { SectionState } from "@/lib/parking/contract";
import { LiveSocket } from "./socket";
import { createLiveStore, type LiveStoreApi } from "./store";

/** How often to re-read `/sections` while the socket is not open. */
const FALLBACK_POLL_MS = 5_000;

const LiveStoreContext = createContext<LiveStoreApi | null>(null);

export function useLiveStoreApi(): LiveStoreApi {
  const store = useContext(LiveStoreContext);

  if (!store) {
    throw new Error("live hooks require a <LiveProvider> above them");
  }

  return store;
}

export interface LiveProviderProps {
  children: ReactNode;
  /**
   * Sections the server already read. Seeding the store from these is what
   * makes the first paint show real availability instead of skeletons.
   */
  initialSections?: readonly SectionState[];
  /** Set when the server-side read failed, so the UI can say so honestly. */
  initialError?: string | null;
}

export function LiveProvider({
  children,
  initialSections = [],
  initialError = null,
}: LiveProviderProps) {
  // `useState` with an initialiser, not `useRef`: the store must be created
  // exactly once per provider and must exist during the very first render, on
  // the server as well as in the browser.
  const [store] = useState(() => createLiveStore(initialSections));
  const socketRef = useRef<LiveSocket | null>(null);

  useEffect(() => {
    if (initialError) store.getState().reportProtocolError(initialError);
  }, [store, initialError]);

  useEffect(() => {
    const { ingest, setPhase, reportProtocolError } = store.getState();

    const socket = new LiveSocket(websocketUrl(), {
      onEvent: ingest,
      onPhase: (phase, info) => setPhase(phase, info),
      onProtocolError: reportProtocolError,
    });

    socketRef.current = socket;
    socket.start();

    const reconnect = () => socket.reconnectNow();
    const onVisible = () => {
      if (document.visibilityState === "visible") socket.reconnectNow();
    };

    window.addEventListener("online", reconnect);
    document.addEventListener("visibilitychange", onVisible);

    return () => {
      window.removeEventListener("online", reconnect);
      document.removeEventListener("visibilitychange", onVisible);
      socket.stop();
      socketRef.current = null;
    };
  }, [store]);

  return (
    <LiveStoreContext.Provider value={store}>
      <SectionsFallback />
      {children}
    </LiveStoreContext.Provider>
  );
}

/**
 * Renders nothing; polls `/sections` only while the socket is not open. Split
 * into its own component so a poll result re-renders the query hook and the
 * store subscribers, never the whole provider subtree.
 */
function SectionsFallback() {
  const store = useLiveStoreApi();
  const [degraded, setDegraded] = useState(false);

  useEffect(() => {
    const evaluate = () => {
      const { phase } = store.getState().connection;

      setDegraded(phase === "reconnecting" || phase === "stopped");
    };

    evaluate();

    return store.subscribe(evaluate);
  }, [store]);

  const { data } = useQuery({
    queryKey: ["sections", "fallback"],
    queryFn: ({ signal }) => listSections({ signal }),
    enabled: degraded,
    refetchInterval: FALLBACK_POLL_MS,
    refetchOnWindowFocus: true,
    // The board's own staleness rules decide what is old; the query layer only
    // has to keep fetching while the socket is down.
    staleTime: 0,
  });

  useEffect(() => {
    if (data) store.getState().ingestSections(data);
  }, [store, data]);

  return null;
}
