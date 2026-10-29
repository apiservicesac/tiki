import { describe, expect, it } from "vitest";
import { CoreDateTimeText } from "../src/backend/CoreDateTimeText";

describe("CoreDateTimeText", () => {
  it("reads the day and the time of a moment", () => {
    expect(CoreDateTimeText.day("2026-10-29T09:30:00")).toBe("2026-10-29");
    expect(CoreDateTimeText.time("2026-10-29T09:30:00")).toBe("09:30");
    expect(CoreDateTimeText.clock("08:30:00")).toBe("08:30");
  });

  it("converts a moment to and from a date-and-time picker", () => {
    expect(CoreDateTimeText.toPicker("2026-10-29T09:30:00")).toBe("2026-10-29T09:30");
    expect(CoreDateTimeText.toPicker(null)).toBe("");
    expect(CoreDateTimeText.fromPicker("2026-10-29T09:30")).toBe("2026-10-29T09:30:00");
    expect(CoreDateTimeText.fromPicker("")).toBeNull();
  });

  it("counts whole days from today", () => {
    const today = new Date(2026, 9, 29, 18, 45);
    expect(CoreDateTimeText.daysFrom(today, "2026-10-29")).toBe(0);
    expect(CoreDateTimeText.daysFrom(today, "2026-10-30")).toBe(1);
    expect(CoreDateTimeText.daysFrom(today, "2026-11-02")).toBe(4);
  });
});
