/**
 * The single HTTP path to the backend.
 *
 * Three jobs, none of which belongs in a component:
 *
 * 1. **Never cache.** Availability is live data; a stale free-slot count is
 *    worse than no count. Every request is `no-store`.
 * 2. **One error type.** Transport failure, a backend error envelope, and a
 *    payload that does not match the contract all surface as `ApiError` with a
 *    machine-readable `code`, so callers branch on one thing. `request_id` is
 *    carried through when the backend echoes it, which is what lets a user
 *    report be matched to a log line.
 * 3. **Validate at the boundary.** Responses are parsed with the schemas in
 *    `parking/contract.ts` before anything downstream sees them.
 */
import type { z } from "zod";

import { type ErrorCode, errorResponseSchema } from "@/lib/parking/contract";

/**
 * Every way a call can fail. The backend's own codes are reused verbatim and
 * two client-side ones are added, because "the server never answered" and "the
 * answer did not match the contract" are distinct problems with distinct fixes.
 */
export type ApiErrorCode = ErrorCode | "network" | "invalid_response";

export class ApiError extends Error {
  readonly code: ApiErrorCode;
  readonly status?: number;
  readonly requestId?: string;

  constructor(
    code: ApiErrorCode,
    message: string,
    options: { status?: number; requestId?: string; cause?: unknown } = {},
  ) {
    super(message, { cause: options.cause });
    this.name = "ApiError";
    this.code = code;
    this.status = options.status;
    this.requestId = options.requestId;
  }

  /** `true` when retrying the same request could plausibly succeed. */
  get retryable(): boolean {
    return (
      this.code === "network" ||
      this.code === "internal" ||
      this.code === "request_timeout" ||
      this.code === "rate_limited" ||
      this.code === "service_unavailable"
    );
  }
}

export interface RequestOptions {
  signal?: AbortSignal;
  /** Client-side deadline; the backend has its own `request_timeout_secs`. */
  timeoutMs?: number;
  /**
   * Statuses to parse as a success even though they are not 2xx. The
   * operational endpoints answer `503` with their *normal* body when a
   * component is unhealthy (`/readyz`, `/status`) — that body is the answer,
   * not an error envelope, so refusing it would throw away the very
   * information the endpoint exists to report.
   */
  expectStatus?: readonly number[];
}

const DEFAULT_TIMEOUT_MS = 8_000;

export async function requestJson<TSchema extends z.ZodType>(
  url: string,
  schema: TSchema,
  options: RequestOptions = {},
): Promise<z.infer<TSchema>> {
  const response = await fetchOrThrow(url, options);
  const requestId = response.headers.get("x-request-id") ?? undefined;
  const body = await readJson(response, requestId);
  const expected =
    response.ok || options.expectStatus?.includes(response.status) === true;

  if (!expected) {
    throw errorFromEnvelope(response.status, body, requestId);
  }

  const parsed = schema.safeParse(body);

  if (!parsed.success) {
    throw new ApiError(
      "invalid_response",
      `${url} returned a payload that does not match the contract: ${describeIssues(parsed.error)}`,
      { status: response.status, requestId, cause: parsed.error },
    );
  }

  return parsed.data;
}

async function fetchOrThrow(
  url: string,
  options: RequestOptions,
): Promise<Response> {
  const timeout = AbortSignal.timeout(options.timeoutMs ?? DEFAULT_TIMEOUT_MS);
  const signal = options.signal
    ? AbortSignal.any([options.signal, timeout])
    : timeout;

  try {
    return await fetch(url, {
      signal,
      cache: "no-store",
      headers: { accept: "application/json" },
    });
  } catch (cause) {
    // A caller-driven abort (navigation, unmount) is not a backend problem, so
    // it is re-thrown untouched and never rendered as an error state.
    if (options.signal?.aborted) throw cause;

    throw new ApiError("network", `could not reach ${url}`, { cause });
  }
}

async function readJson(
  response: Response,
  requestId: string | undefined,
): Promise<unknown> {
  const text = await response.text();

  if (text.length === 0) return null;

  try {
    return JSON.parse(text) as unknown;
  } catch (cause) {
    throw new ApiError(
      response.ok ? "invalid_response" : codeForStatus(response.status),
      `backend returned ${response.status} with a non-JSON body`,
      { status: response.status, requestId, cause },
    );
  }
}

/**
 * Failures are expected to carry the backend's envelope
 * (`{"error":{"code","message"}}`). Anything else — a proxy's HTML error page,
 * a gateway timeout — is mapped by status so the UI still has a usable code.
 */
function errorFromEnvelope(
  status: number,
  body: unknown,
  requestId: string | undefined,
): ApiError {
  const envelope = errorResponseSchema.safeParse(body);

  if (envelope.success) {
    const { code, message, request_id } = envelope.data.error;

    return new ApiError(code, message, {
      status,
      requestId: request_id ?? requestId,
    });
  }

  return new ApiError(codeForStatus(status), `backend returned ${status}`, {
    status,
    requestId,
  });
}

function codeForStatus(status: number): ApiErrorCode {
  switch (status) {
    case 400:
      return "bad_request";
    case 401:
      return "unauthorized";
    case 404:
      return "not_found";
    case 408:
      return "request_timeout";
    case 429:
      return "rate_limited";
    case 503:
      return "service_unavailable";
    default:
      return "internal";
  }
}

function describeIssues(error: z.ZodError): string {
  return error.issues
    .slice(0, 3)
    .map((issue) => `${issue.path.join(".") || "<root>"}: ${issue.message}`)
    .join("; ");
}
