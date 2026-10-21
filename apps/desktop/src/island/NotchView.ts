import type { IslandSnapshot, MoodName } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { MascotRenderer } from "../mascot/MascotRenderer";

/**
 * Content of the folded island. Resting, it shows Tiki with the day's summary when something
 * needs attention, or Tiki alone when the day is clear. Compact adds the mood word.
 */
export class NotchView {
  constructor(private readonly mascot: MascotRenderer) {}

  hidden(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    const mascot = ElementFactory.fromTrustedMarkup(
      "notch-mascot",
      this.mascot.render(mood, snapshot.moodLabel, UI_CONFIG.mascotSize.hidden),
    );
    if (snapshot.attentionCount === 0) {
      return ElementFactory.create("div", { className: "notch-hidden", children: [mascot] });
    }
    return ElementFactory.create("div", {
      className: "notch-hidden notch-hidden-busy",
      children: [
        mascot,
        ElementFactory.create("span", { className: "notch-name", text: "Tiki" }),
        ElementFactory.create("span", { className: "notch-summary", text: snapshot.closedSummary }),
      ],
    });
  }

  compact(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    return ElementFactory.create("div", {
      className: "notch-compact",
      children: [
        ElementFactory.fromTrustedMarkup(
          "notch-mascot",
          this.mascot.render(mood, snapshot.moodLabel, UI_CONFIG.mascotSize.compact),
        ),
        ElementFactory.create("span", { className: "notch-summary", text: snapshot.closedSummary }),
        ElementFactory.create("span", {
          className: `notch-chip notch-chip-${mood}`,
          text: snapshot.moodLabel,
        }),
      ],
    });
  }
}
