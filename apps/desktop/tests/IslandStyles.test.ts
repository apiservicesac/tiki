import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { ISLAND_MODES } from "../src/island/islandState";

// Vitest runs from apps/desktop; CSS imports are stubbed in tests, so read the file itself.
const islandStyles = readFileSync(resolve("src/styles/island.css"), "utf8");

describe("island styles", () => {
  it.each(ISLAND_MODES)("give the %s notch its rounded corners", (mode) => {
    const rule = new RegExp(`\\.island-notch\\[data-mode="${mode}"\\]\\s*\\{[^}]*--radius:`);
    expect(islandStyles).toMatch(rule);
  });
});
