import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { ISLAND_MODES } from "../src/island/islandState";
import { FACE_APPEARANCES } from "../src/mascot/mascotAppearance";

// Vitest runs from apps/desktop; CSS imports are stubbed in tests, so read the file itself.
const islandStyles = readFileSync(resolve("src/styles/island.css"), "utf8");
const tokenStyles = readFileSync(resolve("src/styles/tokens.css"), "utf8");
const faceStyles = readFileSync(resolve("src/styles/face.css"), "utf8");
const controlStyles = readFileSync(resolve("src/styles/controls.css"), "utf8");
const lightTokens = tokenStyles.slice(tokenStyles.indexOf("@media (prefers-color-scheme: light)"));
const MOODS = Object.keys(FACE_APPEARANCES);

describe("island styles", () => {
  it.each(ISLAND_MODES)("give the %s notch its rounded corners", (mode) => {
    const rule = new RegExp(`\\.island-notch\\[data-mode="${mode}"\\]\\s*\\{[^}]*--radius:`);
    expect(islandStyles).toMatch(rule);
  });

  it.each(MOODS)("define the %s mood color once, as a token, and map it to --mood-color", (mood) => {
    expect(tokenStyles).toMatch(new RegExp(`--mood-${mood}:\\s*#`));
    expect(faceStyles).toContain(`[data-mood="${mood}"], .mascot-${mood} { --mood-color: var(--mood-${mood}); }`);
  });

  it("never write a mood color by hand in the island", () => {
    expect(islandStyles).not.toMatch(/data-mood="\w+"\][^{]*\{[^}]*rgba\(/);
  });

  it.each(["--good", "--good-background", "--good-edge", ...["critical", "soon", "waiting", "normal"].map((urgency) => `--urgency-${urgency}-background`)])(
    "redefine %s for the light theme",
    (token) => {
      expect(lightTokens).toMatch(new RegExp(`${token}:`));
    },
  );

  it("take colors, radii and type sizes from the tokens, not from raw values", () => {
    for (const styles of [islandStyles, controlStyles]) {
      expect(styles).not.toMatch(/rgba\(52, 211, 153/);
      expect(styles).not.toMatch(/#fff\b|#ffffff/i);
      expect(styles).not.toMatch(/border-radius:\s*(12|14|20|99|999)px/);
      expect(styles).not.toMatch(/font(-size)?:[^;]*\b(13|12\.5|12|11\.5|11)px/);
    }
  });
});
