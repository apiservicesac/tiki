import { describe, expect, it } from "vitest";
import type { MoodName } from "../src/backend/contracts";
import { MascotRenderer } from "../src/mascot/MascotRenderer";
import { MASCOT_APPEARANCES } from "../src/mascot/mascotAppearance";

describe("MascotRenderer", () => {
  const renderer = new MascotRenderer();

  it("draws a valid SVG for every mood", () => {
    for (const mood of Object.keys(MASCOT_APPEARANCES) as MoodName[]) {
      const markup = renderer.render(mood, "200 OK", 44);
      const svg = new DOMParser().parseFromString(markup, "image/svg+xml").documentElement;
      expect(svg.tagName).toBe("svg");
      expect(svg.getAttribute("class")).toContain(`mascot-${mood}`);
    }
  });

  it("writes the translated word on the ticket tag", () => {
    expect(renderer.render("hurry", "¡HOY!", 44)).toContain("¡HOY!");
  });

  it("escapes the tag text", () => {
    const markup = renderer.render("calm", "<b>", 44);
    expect(markup).toContain("&lt;b&gt;");
    expect(markup).not.toContain("<b>");
  });

  it("shows a badge only for moods that have one", () => {
    expect(renderer.render("curious", "NUEVO", 44)).toContain(">+1<");
    expect(renderer.render("calm", "200 OK", 44)).not.toContain(">+1<");
  });
});
