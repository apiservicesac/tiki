import type { IslandSnapshot, MoodName } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { MascotRenderer } from "../mascot/MascotRenderer";

/** Content of the folded island: Tiki alone (hidden) or Tiki with the day's summary (compact). */
export class NotchView {
  constructor(private readonly mascot: MascotRenderer) {}

  hidden(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    return ElementFactory.create("div", {
      className: "notch-hidden",
      children: [
        ElementFactory.fromTrustedMarkup(
          "notch-mascot",
          this.mascot.render(mood, snapshot.moodLabel, UI_CONFIG.mascotSize.hidden),
        ),
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
