import { describe, expect, it } from "vitest";
import { ItemContext } from "../src/island/ItemContext";

describe("ItemContext", () => {
  it("joins the project and the tags", () => {
    expect(ItemContext.describe("Acme", ["billing", "cierre"])).toBe("Acme · #billing · #cierre");
    expect(ItemContext.describe(null, ["billing"])).toBe("#billing");
    expect(ItemContext.describe("Acme", [])).toBe("Acme");
  });

  it("is empty for an item without project or tags", () => {
    expect(ItemContext.describe(null, [])).toBeNull();
  });
});
