import { describe, expect, it } from "vitest";
import { CoreDateTimeText } from "../src/backend/CoreDateTimeText";

describe("CoreDateTimeText", () => {
  it("reads a time of day for a time picker", () => {
    expect(CoreDateTimeText.clock("08:30:00")).toBe("08:30");
  });

  it("converts a moment to and from a date-and-time picker", () => {
    expect(CoreDateTimeText.toPicker("2026-10-29T09:30:00")).toBe("2026-10-29T09:30");
    expect(CoreDateTimeText.toPicker(null)).toBe("");
    expect(CoreDateTimeText.fromPicker("2026-10-29T09:30")).toBe("2026-10-29T09:30:00");
    expect(CoreDateTimeText.fromPicker("")).toBeNull();
  });
});
