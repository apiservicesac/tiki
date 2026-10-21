import { describe, expect, it } from "vitest";
import { Translator } from "../src/i18n/Translator";

describe("Translator", () => {
  it("reads the shared Spanish and English catalogs", () => {
    expect(new Translator("es").text("island.tab.today")).toBe("Hoy");
    expect(new Translator("en").text("island.tab.today")).toBe("Today");
  });

  it("fills placeholders", () => {
    const summary = new Translator("es").format("island.closed.summary", { due: 3, waiting: 2 });
    expect(summary).toBe("3 hoy · 2 esperando");
  });

  it("returns the key when a message is missing", () => {
    expect(new Translator("en").text("missing.key")).toBe("missing.key");
  });
});
