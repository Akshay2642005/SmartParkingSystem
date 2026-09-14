import { describe, it, expect, vi } from "vitest";
import { LiveSocket } from "@/lib/live/socket";
import type { LiveSocketHooks } from "@/lib/live/socket";
import type { ServerEvent } from "@/lib/parking/contract";

// ---------- minimal WebSocket mock ----------

interface MockWS {
  readyState: number;
  onopen: (() => void) | null;
  onmessage: ((e: { data: unknown }) => void) | null;
  onerror: (() => void) | null;
  onclose: ((e: { code: number; reason: string }) => void) | null;
  send: ReturnType<typeof vi.fn>;
  close: ReturnType<typeof vi.fn>;
  // helpers
  _open(): void;
  _message(data: string): void;
  _drop(code?: number, reason?: string): void;
}

/**
 * Creates a self-contained mock factory. Each call returns a fresh factory
 * so tests are isolated without `vi.clearAllMocks()`.
 *
 * Vitest 4 requires `new`-able mocks to use a `function` declaration, not an
 * arrow or lambda. The inner function captures the test's `instances` array so
 * each test can grab the socket that was actually constructed.
 */
function makeMockFactory() {
  const instances: MockWS[] = [];

  // Must be a `function` declaration (not arrow fn) to satisfy Vitest 4's
  // constructor-mock requirement.
  function FakeWebSocket(this: MockWS) {
    this.readyState = 1; // OPEN
    this.onopen = null;
    this.onmessage = null;
    this.onerror = null;
    this.onclose = null;
    this.send = vi.fn();
    this.close = vi.fn();

    this._open = () => this.onopen?.();
    this._message = (data: string) => this.onmessage?.({ data });
    this._drop = (code = 1006, reason = "") => {
      this.readyState = 3;
      this.onclose?.({ code, reason });
    };

    instances.push(this);
  }

  // Static constants LiveSocket reads when checking readyState.
  FakeWebSocket.CONNECTING = 0;
  FakeWebSocket.OPEN = 1;
  FakeWebSocket.CLOSING = 2;
  FakeWebSocket.CLOSED = 3;

  const last = (): MockWS => {
    const ws = instances[instances.length - 1];
    if (!ws) throw new Error("No WebSocket constructed yet; call socket.start() first.");
    return ws;
  };

  return { MockWebSocket: FakeWebSocket as unknown as typeof WebSocket, last };
}

// ---------- test helpers ----------

const deterministicRandom = () => 0.5;

function makeHooks() {
  const events: ServerEvent[] = [];
  const phases: string[] = [];
  const errors: string[] = [];

  const hooks: LiveSocketHooks = {
    onEvent: (e) => events.push(e),
    onPhase: (p) => phases.push(p),
    onProtocolError: (m) => errors.push(m),
  };

  return { hooks, events, phases, errors };
}

function makeSocket(hooks: LiveSocketHooks, wsFactory?: typeof WebSocket): LiveSocket {
  return new LiveSocket(
    "ws://localhost:8080/api/v1/ws",
    hooks,
    {
      webSocketImpl: wsFactory ?? makeMockFactory().MockWebSocket,
      random: deterministicRandom,
      minBackoffMs: 100,
      maxBackoffMs: 1000,
      setTimeoutImpl: vi.fn().mockReturnValue(99) as unknown as typeof setTimeout,
      clearTimeoutImpl: vi.fn() as unknown as typeof clearTimeout,
    },
  );
}

// ---------- tests ----------

describe("LiveSocket", () => {
  it("reports 'connecting' then 'open'", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, phases } = makeHooks();
    makeSocket(hooks, MockWebSocket).start();
    last()._open();
    expect(phases).toContain("connecting");
    expect(phases).toContain("open");
  });

  it("delivers a parsed snapshot event", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, events } = makeHooks();
    makeSocket(hooks, MockWebSocket).start();
    last()._open();
    last()._message(
      JSON.stringify({
        type: "snapshot",
        sections: [],
        server_ts_ms: 1_700_000_000_000,
      }),
    );
    expect(events).toHaveLength(1);
    expect(events[0]?.type).toBe("snapshot");
  });

  it("reports protocol error for unknown type and does not fire onEvent", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, events, errors } = makeHooks();
    makeSocket(hooks, MockWebSocket).start();
    last()._open();
    last()._message(JSON.stringify({ type: "heartbeat" }));
    expect(errors).toHaveLength(1);
    expect(events).toHaveLength(0);
  });

  it("reports protocol error for non-JSON", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, errors } = makeHooks();
    makeSocket(hooks, MockWebSocket).start();
    last()._open();
    last()._message("not json");
    expect(errors.length).toBeGreaterThan(0);
  });

  it("schedules reconnect on drop", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, phases } = makeHooks();
    const setTimeoutMock = vi.fn().mockReturnValue(99) as unknown as typeof setTimeout;
    const sock = new LiveSocket("ws://localhost:8080/api/v1/ws", hooks, {
      webSocketImpl: MockWebSocket,
      random: deterministicRandom,
      minBackoffMs: 100,
      maxBackoffMs: 1000,
      setTimeoutImpl: setTimeoutMock,
      clearTimeoutImpl: vi.fn() as unknown as typeof clearTimeout,
    });
    sock.start();
    last()._open();
    last()._drop();
    expect(phases).toContain("reconnecting");
    expect(setTimeoutMock).toHaveBeenCalled();
  });

  it("backoffMs is bounded between min and max", () => {
    const { hooks } = makeHooks();
    const sock = makeSocket(hooks);
    for (let attempt = 1; attempt <= 20; attempt++) {
      const ms = sock.backoffMs(attempt);
      expect(ms).toBeGreaterThanOrEqual(75);
      expect(ms).toBeLessThanOrEqual(1250);
    }
  });

  it("stop() closes the socket and reports stopped", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks, phases } = makeHooks();
    const sock = makeSocket(hooks, MockWebSocket);
    sock.start();
    last()._open();
    sock.stop();
    expect(last().close).toHaveBeenCalled();
    expect(phases).toContain("stopped");
  });

  it("send() forwards JSON to the socket", () => {
    const { MockWebSocket, last } = makeMockFactory();
    const { hooks } = makeHooks();
    const sock = makeSocket(hooks, MockWebSocket);
    sock.start();
    last()._open();
    const sent = sock.send({ type: "cmd", name: "reserve" });
    expect(sent).toBe(true);
    expect(last().send).toHaveBeenCalledWith(
      JSON.stringify({ type: "cmd", name: "reserve" }),
    );
  });

  it("send() returns false when socket is not open", () => {
    const { hooks } = makeHooks();
    const sock = makeSocket(hooks);
    expect(sock.send({ type: "cmd" })).toBe(false);
  });
});

