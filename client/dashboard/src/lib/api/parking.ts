/**
 * The backend's read surface, one function per route.
 *
 * These are the only places a URL is written down, so the route table in
 * `backend-architecture.md` § HTTP Surface can be checked against this file at
 * a glance. Callers get parsed, typed data or an `ApiError`.
 *
 * The WebSocket carries the same `SectionState` shape (`ADR-0009`), so these
 * reads and the live stream are interchangeable sources for the same store —
 * which is what makes REST a usable fallback when the socket cannot connect.
 */
import { z } from "zod";

import { requestJson, type RequestOptions } from "./client";
import { apiUrl, opsUrl } from "./endpoints";
import {
  probeResponseSchema,
  sectionStateSchema,
  statusResponseSchema,
} from "@/lib/parking/contract";

const sectionListSchema = z.array(sectionStateSchema);

/** `GET {prefix}/sections` — every section of every site. */
export function listSections(options?: RequestOptions) {
  return requestJson(apiUrl("/sections"), sectionListSchema, options);
}

/** `GET {prefix}/sites/{site}/sections` — 404 when no section has reported. */
export function listSiteSections(site: string, options?: RequestOptions) {
  return requestJson(
    apiUrl(`/sites/${encodeURIComponent(site)}/sections`),
    sectionListSchema,
    options,
  );
}

/**
 * `GET {prefix}/sites/{site}/sections/{section}` — 404 both when the section is
 * unknown and when its node has gone offline, because the backend clears state
 * on a retained `offline`. The two are indistinguishable over REST; the
 * WebSocket's `node_status` frame is what tells them apart.
 */
export function getSection(
  site: string,
  section: string,
  options?: RequestOptions,
) {
  return requestJson(
    apiUrl(
      `/sites/${encodeURIComponent(site)}/sections/${encodeURIComponent(section)}`,
    ),
    sectionStateSchema,
    options,
  );
}

/**
 * `GET /status` — component detail. Unprefixed and never rate limited.
 *
 * Answers 503 with the same body when a component is unhealthy, so a degraded
 * backend still reports *what* is degraded instead of collapsing into a
 * transport error.
 */
export function getStatus(options?: RequestOptions) {
  return requestJson(opsUrl("/status"), statusResponseSchema, {
    ...options,
    expectStatus: [503],
  });
}

/**
 * `GET /readyz` — 200 only while the backend can answer truthfully about
 * parking state (device feed connected *and* store responding). It answers 503
 * with the same body when it cannot, so the response is read, not just the
 * status code.
 */
export function getReadiness(options?: RequestOptions) {
  return requestJson(opsUrl("/readyz"), probeResponseSchema, {
    ...options,
    expectStatus: [503],
  });
}
