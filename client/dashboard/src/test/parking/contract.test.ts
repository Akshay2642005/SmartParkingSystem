import { describe, it, expect } from "vitest";
import {
  slotStateSchema,
  slotSchema,
  sectionStateSchema,
  serverEventSchema,
  sectionKey,
  sectionKeyOf,
} from "@/lib/parking/contract";

const SLOT_FREE = { id: "A-1", state: "free" as const, changed_ms: 0 };
const SLOT_OCC = { id: "A-2", state: "occupied" as const, changed_ms: 100 };
const SLOT_ERR = { id: "A-3", state: "error" as const, changed_ms: 200 };

const SECTION = {
  site: "main",
  section: "A",
  seq: 42,
  slot_count: 3,
  slots: [SLOT_FREE, SLOT_OCC, SLOT_ERR],
  server_ts_ms: 1_700_000_000_000,
};

describe("slotStateSchema", () => {
  it("accepts the three protocol tokens", () => {
    expect(slotStateSchema.parse("free")).toBe("free");
    expect(slotStateSchema.parse("occupied")).toBe("occupied");
    expect(slotStateSchema.parse("error")).toBe("error");
  });

  it("rejects unknown tokens", () => {
    expect(() => slotStateSchema.parse("unknown")).toThrow();
    expect(() => slotStateSchema.parse("")).toThrow();
  });
});

describe("slotSchema", () => {
  it("parses a valid slot", () => {
    const slot = slotSchema.parse(SLOT_FREE);
    expect(slot.id).toBe("A-1");
    expect(slot.state).toBe("free");
  });

  it("rejects an empty id", () => {
    expect(() => slotSchema.parse({ ...SLOT_FREE, id: "" })).toThrow();
  });
});

describe("sectionStateSchema", () => {
  it("parses a valid section", () => {
    const section = sectionStateSchema.parse(SECTION);
    expect(section.site).toBe("main");
    expect(section.slots).toHaveLength(3);
  });

  it("rejects negative server_ts_ms", () => {
    expect(() =>
      sectionStateSchema.parse({ ...SECTION, server_ts_ms: -1 }),
    ).toThrow();
  });
});

describe("serverEventSchema — snapshot", () => {
  it("parses a snapshot frame", () => {
    const event = serverEventSchema.parse({
      type: "snapshot",
      sections: [SECTION],
      server_ts_ms: 1_700_000_000_000,
    });
    expect(event.type).toBe("snapshot");
    if (event.type === "snapshot") {
      expect(event.sections).toHaveLength(1);
    }
  });
});

describe("serverEventSchema — update", () => {
  it("parses an update frame", () => {
    const event = serverEventSchema.parse({
      type: "update",
      section: SECTION,
    });
    expect(event.type).toBe("update");
  });
});

describe("serverEventSchema — node_status", () => {
  it("parses online and offline", () => {
    for (const status of ["online", "offline"] as const) {
      const event = serverEventSchema.parse({
        type: "node_status",
        site: "main",
        section: "A",
        status,
        server_ts_ms: 1_700_000_000_000,
      });
      expect(event.type).toBe("node_status");
    }
  });
});

describe("serverEventSchema — error", () => {
  it("parses an error frame", () => {
    const event = serverEventSchema.parse({
      type: "error",
      code: "service_unavailable",
      message: "store not ready",
    });
    expect(event.type).toBe("error");
    if (event.type === "error") {
      expect(event.code).toBe("service_unavailable");
    }
  });

  it("rejects unknown type discriminant", () => {
    expect(() =>
      serverEventSchema.parse({ type: "heartbeat" }),
    ).toThrow();
  });
});

describe("sectionKey / sectionKeyOf", () => {
  it("produces site/section strings", () => {
    expect(sectionKey("main", "A")).toBe("main/A");
    expect(sectionKeyOf({ site: "main", section: "B" })).toBe("main/B");
  });
});
