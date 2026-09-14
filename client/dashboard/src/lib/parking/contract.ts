/**
 * The wire contract, validated.
 *
 * Every type here mirrors a Rust type the backend serialises, so the two can be
 * read side by side:
 *
 * | This module        | Backend                                    |
 * | ------------------ | ------------------------------------------ |
 * | `SlotState`        | `protocol::SlotState`                      |
 * | `Slot`             | `protocol::Slot`                           |
 * | `SectionState`     | `domain::parking::SectionState`             |
 * | `NodeStatus`       | `domain::parking::NodeStatus`               |
 * | `ServerEvent`      | `events::ServerEvent`                      |
 * | `ErrorCode`        | `response::error::ErrorCode`                |
 * | `StatusResponse`   | `response::system::StatusResponse`          |
 *
 * Field names stay snake_case on purpose: these objects are the backend's
 * payloads, not our own shapes, and keeping the names identical means a spec or
 * Rust struct can be grepped for directly. Types we derive ourselves (see
 * `derive.ts`) use camelCase, which is the visible line between "what the
 * backend said" and "what this dashboard concluded".
 *
 * Parsing is not ceremony. The dashboard is a separate deployable from the
 * backend, so a version skew shows up here as a typed, reportable failure
 * instead of `undefined` reaching a chart.
 *
 * Source of truth: `docs/specs/architecture/communication.md`,
 * `docs/specs/architecture/backend-architecture.md`, `ADR-0009`.
 */
import { z } from "zod";

/** Slot occupancy, protocol v1: the only three tokens a node may publish. */
export const slotStateSchema = z.enum(["free", "occupied", "error"]);
export type SlotState = z.infer<typeof slotStateSchema>;

export const slotSchema = z.object({
  /** Full slot label including the section prefix, e.g. `A-1`. */
  id: z.string().min(1),
  state: slotStateSchema,
  /** Node uptime (ms) at the last observed transition. Not wall clock. */
  changed_ms: z.number().nonnegative(),
});
export type Slot = z.infer<typeof slotSchema>;

/**
 * The latest accepted snapshot of one section. The backend uses this single
 * type as both its stored record and its read model, so REST reads and
 * WebSocket frames carry exactly this shape.
 */
export const sectionStateSchema = z.object({
  site: z.string().min(1),
  section: z.string().min(1),
  /** Per-node counter; restarts at 1 after a node reboot. */
  seq: z.number().nonnegative(),
  /** Learned from the section's first accepted snapshot, then enforced. */
  slot_count: z.number().nonnegative(),
  slots: z.array(slotSchema),
  /** Wall clock (ms) at which the backend accepted the snapshot. */
  server_ts_ms: z.number().nonnegative(),
});
export type SectionState = z.infer<typeof sectionStateSchema>;

/** Node liveness, published retained by the node's MQTT will. */
export const nodeStatusSchema = z.enum(["online", "offline"]);
export type NodeStatus = z.infer<typeof nodeStatusSchema>;

/**
 * The stable half of the error envelope. `message` is for humans and may
 * change; `code` is the contract a client is allowed to branch on.
 */
export const errorCodeSchema = z.enum([
  "internal",
  "not_found",
  "bad_request",
  "request_timeout",
  "rate_limited",
  "unauthorized",
  "service_unavailable",
  "commands_not_supported",
  "invalid_frame",
]);
export type ErrorCode = z.infer<typeof errorCodeSchema>;

export const errorResponseSchema = z.object({
  error: z.object({
    code: errorCodeSchema,
    message: z.string(),
    request_id: z.string().optional(),
  }),
});
export type ErrorResponse = z.infer<typeof errorResponseSchema>;

/**
 * Frames the backend pushes over `{prefix}/ws`.
 *
 * The order is contractual (`ADR-0009`): `snapshot` first and exactly once per
 * connection, then `update` and `node_status` as they happen. `error` answers a
 * client frame or reports that parking state is momentarily unreadable; it is
 * never a reason to give up on the socket.
 */
export const serverEventSchema = z.discriminatedUnion("type", [
  z.object({
    type: z.literal("snapshot"),
    sections: z.array(sectionStateSchema),
    server_ts_ms: z.number().nonnegative(),
  }),
  z.object({
    type: z.literal("update"),
    section: sectionStateSchema,
  }),
  z.object({
    type: z.literal("node_status"),
    site: z.string().min(1),
    section: z.string().min(1),
    status: nodeStatusSchema,
    server_ts_ms: z.number().nonnegative(),
  }),
  z.object({
    type: z.literal("error"),
    code: errorCodeSchema,
    message: z.string(),
  }),
]);
export type ServerEvent = z.infer<typeof serverEventSchema>;
export type ServerEventType = ServerEvent["type"];

/**
 * Client frames. v1 answers every command with
 * `{type:"error", code:"commands_not_supported"}` and keeps the socket open;
 * the envelope is reserved so reserve/gate-control can ship without a protocol
 * break. Kept here because the dashboard is the client the reservation is for.
 */
export const clientFrameSchema = z.object({
  type: z.literal("cmd"),
  name: z.string().optional(),
});
export type ClientFrame = z.infer<typeof clientFrameSchema>;

export const probeResponseSchema = z.object({
  status: z.string(),
  timestamp: z.string(),
});
export type ProbeResponse = z.infer<typeof probeResponseSchema>;

export const componentStatusSchema = z.object({
  status: z.string(),
  detail: z.string().optional(),
});
export type ComponentStatus = z.infer<typeof componentStatusSchema>;

/** `GET /status`: the unprefixed, unthrottled component detail endpoint. */
export const statusResponseSchema = z.object({
  status: z.string(),
  service: z.string(),
  version: z.string(),
  environment: z.string(),
  timestamp: z.string(),
  uptime_secs: z.number().nonnegative(),
  /** Sections that have reported at least once and are still online. */
  sections: z.number().nonnegative(),
  /** Dashboard WebSocket clients currently attached, this one included. */
  subscribers: z.number().nonnegative(),
  checks: z.object({
    ingest: componentStatusSchema,
    store: componentStatusSchema,
  }),
});
export type StatusResponse = z.infer<typeof statusResponseSchema>;

/**
 * Identity of a section, `site/section`, matching the backend's `(site,
 * section)` key and the `parking/{site}/{section}/…` topic layout. One function
 * so every map, list, and route agrees on how a section is addressed.
 */
export function sectionKey(site: string, section: string): string {
  return `${site}/${section}`;
}

export function sectionKeyOf(state: {
  site: string;
  section: string;
}): string {
  return sectionKey(state.site, state.section);
}
