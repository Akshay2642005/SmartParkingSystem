/**
 * The dashboard WebSocket, as a transport with no opinions about React.
 *
 * Responsibilities are deliberately narrow: hold one socket open, decode and
 * validate frames, and reconnect with jittered exponential backoff. Everything
 * about what a frame *means* lives in `reducer.ts`.
 *
 * Reconnecting is the whole recovery story and it is cheap by design: the
 * backend closes any client that falls more than 256 events behind precisely so
 * that the client comes back and re-syncs from a fresh snapshot (`ADR-0009`).
 * A drop is therefore normal operation, not an incident — which is why the
 * first retry is fast and the ceiling is low.
 *
 * Timers, randomness, and the `WebSocket` constructor are all injectable so the
 * backoff schedule can be asserted in tests without waiting for real seconds.
 */
import {
  type ServerEvent,
  serverEventSchema,
} from "@/lib/parking/contract";
import type { ConnectionPhase } from "./reducer";

export interface LiveSocketHooks {
  onEvent(event: ServerEvent): void;
  onPhase(
    phase: ConnectionPhase,
    info: { attempt: number; detail?: string },
  ): void;
  /**
   * A frame arrived that is not a valid `ServerEvent`. Surfaced rather than
   * swallowed: it means the dashboard and backend disagree about the contract,
   * which is a deployment problem an operator should see.
   */
  onProtocolError(message: string): void;
}

export interface LiveSocketOptions {
  minBackoffMs?: number;
  maxBackoffMs?: number;
  /** Injected in tests; defaults to the global `WebSocket`. */
  webSocketImpl?: typeof WebSocket;
  random?: () => number;
  setTimeoutImpl?: typeof setTimeout;
  clearTimeoutImpl?: typeof clearTimeout;
}

const DEFAULT_MIN_BACKOFF_MS = 500;
const DEFAULT_MAX_BACKOFF_MS = 15_000;

export class LiveSocket {
  private readonly url: string;
  private readonly hooks: LiveSocketHooks;
  private readonly minBackoffMs: number;
  private readonly maxBackoffMs: number;
  private readonly WebSocketImpl: typeof WebSocket;
  private readonly random: () => number;
  private readonly setTimeoutImpl: typeof setTimeout;
  private readonly clearTimeoutImpl: typeof clearTimeout;

  private socket: WebSocket | null = null;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private attempt = 0;
  private running = false;

  constructor(
    url: string,
    hooks: LiveSocketHooks,
    options: LiveSocketOptions = {},
  ) {
    this.url = url;
    this.hooks = hooks;
    this.minBackoffMs = options.minBackoffMs ?? DEFAULT_MIN_BACKOFF_MS;
    this.maxBackoffMs = options.maxBackoffMs ?? DEFAULT_MAX_BACKOFF_MS;
    this.WebSocketImpl = options.webSocketImpl ?? globalThis.WebSocket;
    this.random = options.random ?? Math.random;
    this.setTimeoutImpl = options.setTimeoutImpl ?? setTimeout;
    this.clearTimeoutImpl = options.clearTimeoutImpl ?? clearTimeout;
  }

  start(): void {
    if (this.running) return;

    this.running = true;
    this.open();
  }

  stop(): void {
    this.running = false;
    this.cancelRetry();
    this.closeSocket();
    this.hooks.onPhase("stopped", { attempt: this.attempt });
  }

  /**
   * Retry immediately instead of waiting out the backoff. Called when the
   * browser reports it is back online or the tab becomes visible again: those
   * are direct evidence that the previous failure no longer applies, so
   * continuing to wait would only add latency the user can see.
   */
  reconnectNow(): void {
    if (!this.running) return;
    if (this.socket && this.socket.readyState === this.WebSocketImpl.OPEN) {
      return;
    }

    this.cancelRetry();
    this.closeSocket();
    this.attempt = 0;
    this.open();
  }

  /**
   * Send a client frame. v1 gets back
   * `{type:"error", code:"commands_not_supported"}` and the socket stays open;
   * this exists so the reserved command envelope has one caller when it does
   * become real, and so the typed rejection is observable from the UI.
   */
  send(frame: { type: "cmd"; name?: string }): boolean {
    if (!this.socket || this.socket.readyState !== this.WebSocketImpl.OPEN) {
      return false;
    }

    this.socket.send(JSON.stringify(frame));

    return true;
  }

  private open(): void {
    this.hooks.onPhase(this.attempt === 0 ? "connecting" : "reconnecting", {
      attempt: this.attempt,
    });

    let socket: WebSocket;

    try {
      socket = new this.WebSocketImpl(this.url);
    } catch (cause) {
      this.scheduleRetry(
        cause instanceof Error ? cause.message : "socket could not be created",
      );

      return;
    }

    this.socket = socket;

    socket.onopen = () => {
      this.attempt = 0;
      this.hooks.onPhase("open", { attempt: 0 });
    };

    socket.onmessage = (message: MessageEvent) => {
      this.receive(message.data);
    };

    // `onerror` carries no useful detail in browsers and is always followed by
    // `onclose`, so retry scheduling lives in one place: `onclose`.
    socket.onerror = () => {};

    socket.onclose = (event: CloseEvent) => {
      if (this.socket !== socket) return;

      this.socket = null;

      if (!this.running) return;

      this.scheduleRetry(describeClose(event));
    };
  }

  private receive(data: unknown): void {
    if (typeof data !== "string") {
      this.hooks.onProtocolError(
        "expected a text frame; the contract is JSON over text frames",
      );

      return;
    }

    let json: unknown;

    try {
      json = JSON.parse(data);
    } catch {
      this.hooks.onProtocolError("frame was not valid JSON");

      return;
    }

    const parsed = serverEventSchema.safeParse(json);

    if (!parsed.success) {
      this.hooks.onProtocolError(
        `frame does not match the event contract: ${parsed.error.issues[0]?.message ?? "unknown field"}`,
      );

      return;
    }

    this.hooks.onEvent(parsed.data);
  }

  private scheduleRetry(detail: string): void {
    this.attempt += 1;

    const delay = this.backoffMs(this.attempt);

    this.hooks.onPhase("reconnecting", { attempt: this.attempt, detail });

    this.retryTimer = this.setTimeoutImpl(() => {
      this.retryTimer = null;

      if (this.running) this.open();
    }, delay);
  }

  /** `min * 2^(attempt-1)`, capped, then ±25 % jitter to avoid a thundering herd. */
  backoffMs(attempt: number): number {
    const exponential = this.minBackoffMs * 2 ** Math.max(0, attempt - 1);
    const capped = Math.min(this.maxBackoffMs, exponential);

    return Math.round(capped * (0.75 + this.random() * 0.5));
  }

  private cancelRetry(): void {
    if (this.retryTimer !== null) {
      this.clearTimeoutImpl(this.retryTimer);
      this.retryTimer = null;
    }
  }

  private closeSocket(): void {
    const socket = this.socket;

    if (!socket) return;

    this.socket = null;
    socket.onopen = null;
    socket.onmessage = null;
    socket.onerror = null;
    socket.onclose = null;

    if (
      socket.readyState === this.WebSocketImpl.OPEN ||
      socket.readyState === this.WebSocketImpl.CONNECTING
    ) {
      socket.close();
    }
  }
}

function describeClose(event: CloseEvent): string {
  if (event.reason) return `closed: ${event.reason} (${event.code})`;

  // 1006 is the browser's "abnormal closure" catch-all: no close handshake
  // happened, which is what a refused connection or a dropped network looks
  // like from script.
  return event.code === 1006
    ? "connection lost"
    : `closed with code ${event.code}`;
}
