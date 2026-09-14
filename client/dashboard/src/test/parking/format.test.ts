import { describe, it, expect } from "vitest";
import {
  formatAge,
  formatAgo,
  formatClockUtc,
  formatTimestampUtc,
  formatPercent,
  formatCount,
} from "@/lib/parking/format";

describe("formatAge", () => {
  it("returns 'now' for 0 ms", () => expect(formatAge(0)).toBe("now"));
  it("returns 'now' for sub-second", () => expect(formatAge(999)).toBe("now"));
  it("formats seconds", () => expect(formatAge(15_000)).toBe("15s"));
  it("formats minutes and seconds", () =>
    expect(formatAge(4 * 60_000 + 20_000)).toBe("4m 20s"));
  it("formats hours and minutes", () =>
    expect(formatAge(3 * 3_600_000 + 5 * 60_000)).toBe("3h 05m"));
  it("formats days and hours", () =>
    expect(formatAge(2 * 86_400_000 + 4 * 3_600_000)).toBe("2d 04h"));
  it("handles Infinity", () => expect(formatAge(Infinity)).toBe("—"));
});

describe("formatAgo", () => {
  it("appends 'ago'", () => expect(formatAgo(15_000)).toBe("15s ago"));
  it("says 'just now' for sub-second", () =>
    expect(formatAgo(500)).toBe("just now"));
});

describe("formatClockUtc", () => {
  // 2026-08-27T14:03:07Z
  const ts = Date.UTC(2026, 7, 27, 14, 3, 7);
  it("formats HH:MM:SSZ", () => expect(formatClockUtc(ts)).toBe("14:03:07Z"));
  it("returns em-dash for NaN", () => expect(formatClockUtc(NaN)).toBe("—"));
});

describe("formatTimestampUtc", () => {
  const ts = Date.UTC(2026, 7, 27, 14, 3, 7);
  it("formats YYYY-MM-DD HH:MM:SSZ", () =>
    expect(formatTimestampUtc(ts)).toBe("2026-08-27 14:03:07Z"));
});

describe("formatPercent", () => {
  it("rounds to nearest integer", () =>
    expect(formatPercent(0.666)).toBe("67%"));
  it("returns em-dash for null", () => expect(formatPercent(null)).toBe("—"));
  it("handles 0 and 1", () => {
    expect(formatPercent(0)).toBe("0%");
    expect(formatPercent(1)).toBe("100%");
  });
});

describe("formatCount", () => {
  it("adds thousands separator", () =>
    expect(formatCount(1_234_567)).toBe("1,234,567"));
  it("leaves small numbers alone", () => expect(formatCount(42)).toBe("42"));
});
