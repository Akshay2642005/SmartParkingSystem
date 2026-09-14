import { describe, it, expect } from "vitest";
import {
  applyServerEvent,
  applyRestSnapshot,
  applyConnectionPhase,
  applyProtocolError,
  initialLiveState,
} from "@/lib/live/reducer";
import type { SectionState } from "@/lib/parking/contract";

const NOW = 1_700_000_000_000;

function section(
  site = "main",
  sec = "A",
  overrides: Partial<SectionState> = {},
): SectionState {
  return {
    site,
    section: sec,
    seq: 1,
    slot_count: 3,
    slots: [
      { id: `${sec}-1`, state: "free", changed_ms: 0 },
      { id: `${sec}-2`, state: "occupied", changed_ms: 10 },
      { id: `${sec}-3`, state: "free", changed_ms: 20 },
    ],
    server_ts_ms: NOW,
    ...overrides,
  };
}

describe("snapshot frame", () => {
  it("seeds the entries map", () => {
    const state = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    expect(Object.keys(state.entries)).toContain("main/A");
    expect(state.entries["main/A"]?.state?.section).toBe("A");
  });

  it("drops sections not in the new snapshot", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const withB = applyServerEvent(
      withA,
      { type: "snapshot", sections: [section("main", "B")], server_ts_ms: NOW },
      NOW,
    );
    expect(Object.keys(withB.entries)).not.toContain("main/A");
    expect(Object.keys(withB.entries)).toContain("main/B");
  });

  it("retains offline entries across snapshots", () => {
    // A went offline, then a snapshot arrives with only B
    const afterOffline = applyServerEvent(
      applyServerEvent(
        initialLiveState(),
        { type: "snapshot", sections: [section()], server_ts_ms: NOW },
        NOW,
      ),
      {
        type: "node_status",
        site: "main",
        section: "A",
        status: "offline",
        server_ts_ms: NOW,
      },
      NOW,
    );
    const afterSnapshot = applyServerEvent(
      afterOffline,
      {
        type: "snapshot",
        sections: [section("main", "B")],
        server_ts_ms: NOW,
      },
      NOW,
    );
    // A is kept (dead-node visibility) but with no state
    expect("main/A" in afterSnapshot.entries).toBe(true);
    expect(afterSnapshot.entries["main/A"]?.state).toBeNull();
    expect(afterSnapshot.entries["main/A"]?.nodeStatus).toBe("offline");
  });

  it("appends a snapshot activity entry", () => {
    const state = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    expect(state.activity[0]?.kind).toBe("snapshot");
  });

  it("samples the timeline", () => {
    const state = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    expect(state.timeline.length).toBeGreaterThan(0);
  });
});

describe("update frame", () => {
  it("replaces the section entry", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const updated = applyServerEvent(
      withA,
      {
        type: "update",
        section: section("main", "A", { seq: 2 }),
      },
      NOW + 1000,
    );
    expect(updated.entries["main/A"]?.state?.seq).toBe(2);
  });

  it("logs slot transitions", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const updated = applyServerEvent(
      withA,
      {
        type: "update",
        section: section("main", "A", {
          seq: 2,
          slots: [
            { id: "A-1", state: "occupied", changed_ms: 50 }, // changed
            { id: "A-2", state: "occupied", changed_ms: 10 },
            { id: "A-3", state: "free", changed_ms: 20 },
          ],
        }),
      },
      NOW + 1000,
    );
    const slotEntries = updated.activity.filter((e) => e.kind === "slot");
    expect(slotEntries).toHaveLength(1);
    if (slotEntries[0]?.kind === "slot") {
      expect(slotEntries[0].slotId).toBe("A-1");
      expect(slotEntries[0].from).toBe("free");
      expect(slotEntries[0].to).toBe("occupied");
    }
  });

  it("does not log unchanged refreshes", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const refreshed = applyServerEvent(
      withA,
      { type: "update", section: section("main", "A", { seq: 2 }) },
      NOW + 30_000,
    );
    const slotEntries = refreshed.activity.filter((e) => e.kind === "slot");
    expect(slotEntries).toHaveLength(0);
  });
});

describe("node_status frame", () => {
  it("sets nodeStatus to offline and clears state", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const offline = applyServerEvent(
      withA,
      {
        type: "node_status",
        site: "main",
        section: "A",
        status: "offline",
        server_ts_ms: NOW,
      },
      NOW,
    );
    expect(offline.entries["main/A"]?.nodeStatus).toBe("offline");
    expect(offline.entries["main/A"]?.state).toBeNull();
  });

  it("logs the status change", () => {
    const withA = applyServerEvent(
      initialLiveState(),
      { type: "snapshot", sections: [section()], server_ts_ms: NOW },
      NOW,
    );
    const offline = applyServerEvent(
      withA,
      {
        type: "node_status",
        site: "main",
        section: "A",
        status: "offline",
        server_ts_ms: NOW,
      },
      NOW,
    );
    expect(offline.activity.some((e) => e.kind === "node")).toBe(true);
  });

  it("does not log unchanged status", () => {
    const base = applyServerEvent(
      applyServerEvent(
        initialLiveState(),
        { type: "snapshot", sections: [section()], server_ts_ms: NOW },
        NOW,
      ),
      {
        type: "node_status",
        site: "main",
        section: "A",
        status: "offline",
        server_ts_ms: NOW,
      },
      NOW,
    );
    const count = base.activity.filter((e) => e.kind === "node").length;
    // second identical offline frame
    const again = applyServerEvent(
      base,
      {
        type: "node_status",
        site: "main",
        section: "A",
        status: "offline",
        server_ts_ms: NOW,
      },
      NOW + 1000,
    );
    expect(again.activity.filter((e) => e.kind === "node").length).toBe(count);
  });
});

describe("REST snapshot", () => {
  it("seeds sections like a socket snapshot", () => {
    const state = applyRestSnapshot(
      initialLiveState(),
      [section()],
      NOW,
    );
    expect(state.entries["main/A"]?.state?.seq).toBe(1);
    expect(state.source).toBe("rest");
  });

  it("does not overwrite socket source when socket is open", () => {
    const withSocket = applyConnectionPhase(
      applyServerEvent(
        initialLiveState(),
        { type: "snapshot", sections: [section()], server_ts_ms: NOW },
        NOW,
      ),
      "open",
      NOW,
    );
    const afterPoll = applyRestSnapshot(
      withSocket,
      [section()],
      NOW + 1000,
    );
    expect(afterPoll.source).toBe("socket");
  });
});

describe("connection phase", () => {
  it("logs a connection entry on phase change", () => {
    const state = applyConnectionPhase(
      initialLiveState(),
      "connecting",
      NOW,
    );
    expect(state.activity[0]?.kind).toBe("connection");
  });

  it("resets attempt counter on open", () => {
    const reconnecting = applyConnectionPhase(
      initialLiveState(),
      "reconnecting",
      NOW,
      { attempt: 3 },
    );
    const opened = applyConnectionPhase(reconnecting, "open", NOW + 1000);
    expect(opened.connection.attempt).toBe(0);
    expect(opened.connection.lastError).toBeNull();
  });
});

describe("protocol error", () => {
  it("appends an error activity entry", () => {
    const state = applyProtocolError(
      initialLiveState(),
      "frame was not valid JSON",
      NOW,
    );
    expect(state.activity[0]?.kind).toBe("error");
    if (state.activity[0]?.kind === "error") {
      expect(state.activity[0].code).toBe("transport");
    }
  });
});

describe("activity feed cap", () => {
  it("never exceeds ACTIVITY_LIMIT", () => {
    let state = initialLiveState();
    for (let i = 0; i < 400; i++) {
      state = applyServerEvent(
        state,
        {
          type: "node_status",
          site: "main",
          section: `S${i}`,
          status: i % 2 === 0 ? "online" : "offline",
          server_ts_ms: NOW,
        },
        NOW + i,
      );
    }
    expect(state.activity.length).toBeLessThanOrEqual(300);
  });
});
