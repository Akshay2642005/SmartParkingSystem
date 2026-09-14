import { describe, it, expect } from "vitest";
import {
  countSlots,
  occupancyOf,
  sectionHealth,
  summariseSite,
  diffSlots,
  compareSections,
  STALE_AFTER_MS,
  toSectionView,
} from "@/lib/parking/derive";
import type { SectionState } from "@/lib/parking/contract";

const NOW = 1_700_000_000_000;

function section(
  overrides: Partial<SectionState> = {},
): SectionState {
  return {
    site: "main",
    section: "A",
    seq: 1,
    slot_count: 3,
    slots: [
      { id: "A-1", state: "free", changed_ms: 0 },
      { id: "A-2", state: "occupied", changed_ms: 10 },
      { id: "A-3", state: "free", changed_ms: 20 },
    ],
    server_ts_ms: NOW,
    ...overrides,
  };
}

describe("countSlots", () => {
  it("counts each state correctly", () => {
    const t = countSlots(section().slots);
    expect(t).toEqual({ free: 2, occupied: 1, error: 0, total: 3 });
  });

  it("counts error slots", () => {
    const t = countSlots([
      { id: "A-1", state: "error", changed_ms: 0 },
    ]);
    expect(t.error).toBe(1);
    expect(t.total).toBe(1);
  });
});

describe("occupancyOf", () => {
  it("excludes error slots from denominator", () => {
    // 1 occupied, 1 free, 1 error => 1/2 = 0.5
    const t = countSlots([
      { id: "A-1", state: "occupied", changed_ms: 0 },
      { id: "A-2", state: "free", changed_ms: 0 },
      { id: "A-3", state: "error", changed_ms: 0 },
    ]);
    expect(occupancyOf(t)).toBeCloseTo(0.5);
  });

  it("returns null when all slots are faulty", () => {
    const t = countSlots([
      { id: "A-1", state: "error", changed_ms: 0 },
    ]);
    expect(occupancyOf(t)).toBeNull();
  });
});

describe("sectionHealth", () => {
  it("is live when state is recent", () => {
    expect(sectionHealth(section(), null, NOW + 1000)).toBe("live");
  });

  it("is stale when snapshot is old", () => {
    expect(
      sectionHealth(section(), null, NOW + STALE_AFTER_MS + 1),
    ).toBe("stale");
  });

  it("is offline when nodeStatus is offline regardless of state", () => {
    expect(sectionHealth(section(), "offline", NOW)).toBe("offline");
  });

  it("is unreported when there is no state and node is not offline", () => {
    expect(sectionHealth(null, null, NOW)).toBe("unreported");
    expect(sectionHealth(null, "online", NOW)).toBe("unreported");
  });
});

describe("diffSlots", () => {
  it("detects a state transition", () => {
    const prev = section();
    const next = section({
      slots: [
        { id: "A-1", state: "occupied", changed_ms: 50 }, // changed
        { id: "A-2", state: "occupied", changed_ms: 10 },
        { id: "A-3", state: "free", changed_ms: 20 },
      ],
    });
    const diff = diffSlots(prev, next);
    expect(diff).toHaveLength(1);
    expect(diff[0]).toMatchObject({ slotId: "A-1", from: "free", to: "occupied" });
  });

  it("returns empty when nothing changed", () => {
    const s = section();
    expect(diffSlots(s, s)).toHaveLength(0);
  });

  it("returns empty when previous is null (first snapshot)", () => {
    expect(diffSlots(null, section())).toHaveLength(0);
  });
});

describe("summariseSite", () => {
  it("sums totals across sections", () => {
    const views = [
      toSectionView("main/A", section(), null, NOW),
      toSectionView("main/B", section(), null, NOW),
    ];
    const totals = summariseSite(views);
    expect(totals.free).toBe(4);
    expect(totals.occupied).toBe(2);
    expect(totals.total).toBe(6);
    expect(totals.sections).toBe(2);
    expect(totals.live).toBe(2);
  });

  it("counts offline sections", () => {
    const views = [
      toSectionView("main/A", null, "offline", NOW),
    ];
    const totals = summariseSite(views);
    expect(totals.offline).toBe(1);
    expect(totals.total).toBe(0);
  });
});

describe("compareSections", () => {
  it("sorts by site then section numerically", () => {
    const views = [
      toSectionView("main/B", section({ section: "B" }), null, NOW),
      toSectionView("main/A", section({ section: "A" }), null, NOW),
      toSectionView("main/C", section({ section: "C" }), null, NOW),
    ];
    const sorted = [...views].sort(compareSections);
    expect(sorted.map((v) => v.section)).toEqual(["A", "B", "C"]);
  });
});
