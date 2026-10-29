import { describe, expect, it } from "vitest";
import type { MoodName } from "../src/backend/contracts";
import { MascotRenderer } from "../src/mascot/MascotRenderer";
import { FACE_APPEARANCES } from "../src/mascot/mascotAppearance";

describe("MascotRenderer", () => {
  const renderer = new MascotRenderer();
  const parse = (markup: string) => new DOMParser().parseFromString(markup, "image/svg+xml").documentElement;

  it("draws a valid face for every mood, with its motion", () => {
    for (const mood of Object.keys(FACE_APPEARANCES) as MoodName[]) {
      const svg = parse(renderer.render(mood, "TODO BIEN", 44));
      expect(svg.tagName).toBe("svg");
      expect(svg.getAttribute("class")).toContain(`mascot-${mood}`);
      expect(svg.getAttribute("data-motion")).toBe(FACE_APPEARANCES[mood].motion);
      expect(svg.querySelectorAll(".mascot-eye")).toHaveLength(2);
    }
  });

  it("is twice as wide as it is tall", () => {
    const svg = parse(renderer.render("calm", "", 40));
    expect(svg.getAttribute("width")).toBe("40");
    expect(svg.getAttribute("height")).toBe("20");
  });

  it("paints the eyes with the mood color from the stylesheet", () => {
    const markup = renderer.render("worried", "", 44);
    expect(markup).toContain("currentColor");
    expect(markup).not.toMatch(/#[0-9a-f]{6}/i);
  });

  it("names the mood for screen readers and escapes it", () => {
    const svg = parse(renderer.render("calm", "<b>", 44));
    expect(svg.getAttribute("aria-label")).toBe("<b>");
    expect(renderer.render("calm", "<b>", 44)).not.toContain("<b>");
  });

  it("hides a face without a label from screen readers", () => {
    expect(parse(renderer.render("calm", "", 44)).getAttribute("aria-hidden")).toBe("true");
  });
});
