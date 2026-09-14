/**
 * Where the backend is, resolved once.
 *
 * The backend mounts two kinds of route (`backend-architecture.md` § HTTP
 * Surface) and the difference matters to a client:
 *
 * - **versioned API** under `server.path_prefix` (`/api/v1` by default) — the
 *   parking reads and the dashboard WebSocket; rate limited per IP;
 * - **operational endpoints** with no prefix (`/status`, `/readyz`, …) — never
 *   throttled, shape may change to suit ops.
 *
 * Environment variables:
 *
 * | Variable                        | Used by | Default                 |
 * | ------------------------------- | ------- | ----------------------- |
 * | `NEXT_PUBLIC_PARKING_API_URL`   | browser and server | `http://localhost:8080` |
 * | `PARKING_API_URL`               | server only, wins when set | — |
 * | `NEXT_PUBLIC_PARKING_API_PREFIX`| both    | `/api/v1`               |
 * | `NEXT_PUBLIC_PARKING_WS_URL`    | browser | derived from the API URL |
 *
 * `PARKING_API_URL` exists because server-side rendering may reach the backend
 * over a private address (`http://parking-server:8080` in Compose) that a
 * browser cannot resolve, while the browser needs the public one. Everything
 * else is shared.
 */

const DEFAULT_API_URL = "http://localhost:8080";
const DEFAULT_PREFIX = "/api/v1";

/** Base URL for requests issued from this runtime, without a trailing slash. */
export function apiBaseUrl(): string {
  const serverOverride =
    typeof window === "undefined" ? process.env.PARKING_API_URL : undefined;

  return trimSlash(
    serverOverride ||
      process.env.NEXT_PUBLIC_PARKING_API_URL ||
      DEFAULT_API_URL,
  );
}

/** The backend's `server.path_prefix`, normalised to `/api/v1` form. */
export function apiPrefix(): string {
  const prefix = process.env.NEXT_PUBLIC_PARKING_API_PREFIX || DEFAULT_PREFIX;

  return trimSlash(prefix.startsWith("/") ? prefix : `/${prefix}`);
}

/** A versioned API path: rate limited, part of the dashboard contract. */
export function apiUrl(path: string): string {
  return `${apiBaseUrl()}${apiPrefix()}${ensureLeadingSlash(path)}`;
}

/** An unprefixed operational path: probes, `/status`, `/metrics`. */
export function opsUrl(path: string): string {
  return `${apiBaseUrl()}${ensureLeadingSlash(path)}`;
}

/**
 * The dashboard WebSocket. Derived from the API URL by swapping the scheme so
 * one variable configures both transports; override only when the socket is
 * terminated somewhere else.
 */
export function websocketUrl(): string {
  const configured = process.env.NEXT_PUBLIC_PARKING_WS_URL;

  if (configured) return configured;

  const base = trimSlash(
    process.env.NEXT_PUBLIC_PARKING_API_URL || DEFAULT_API_URL,
  );

  return `${base.replace(/^http/, "ws")}${apiPrefix()}/ws`;
}

function trimSlash(value: string): string {
  return value.replace(/\/+$/, "");
}

function ensureLeadingSlash(path: string): string {
  return path.startsWith("/") ? path : `/${path}`;
}
