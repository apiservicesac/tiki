import { describe, expect, it } from "vitest";
import { ItemContext } from "../src/island/ItemContext";

describe("ItemContext", () => {
  it("joins the project and the tags", () => {
    expect(ItemContext.describe("Altamira", ["billing", "cierre"])).toBe("Altamira · #billing · #cierre");
    expect(ItemContext.describe(null, ["billing"])).toBe("#billing");
    expect(ItemContext.describe("Altamira", [])).toBe("Altamira");
  });

  it("is empty for an item without project or tags", () => {
    expect(ItemContext.describe(null, [])).toBeNull();
  });
});
