import type { IslandSnapshot, MoodName } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { MascotRenderer } from "../mascot/MascotRenderer";

/**
 * Content of the folded island. Resting, it shows Tiki with the day's summary when something
 * needs attention, or Tiki alone when the day is clear. Compact adds the mood word.
 *
 * The summary is written as text for the top and bottom edges and as count bubbles for the
 * left and right edges; the stylesheet shows the one that fits the orientation.
 */
export class NotchView {
  constructor(private readonly mascot: MascotRenderer) {}

  hidden(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    const mascot = this.mascotOf(snapshot, mood, UI_CONFIG.mascotSize.hidden);
    if (snapshot.attentionCount === 0) {
      return ElementFactory.create("div", { className: "notch-hidden", children: [mascot] });
    }
    return ElementFactory.create("div", {
      className: "notch-hidden notch-hidden-busy",
      children: [
        mascot,
        ElementFactory.create("span", { className: "notch-name", text: "Tiki" }),
        this.summaryText(snapshot),
        this.countBubbles(snapshot),
      ],
    });
  }

  compact(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    return ElementFactory.create("div", {
      className: "notch-compact",
      children: [
        this.mascotOf(snapshot, mood, UI_CONFIG.mascotSize.compact),
        this.summaryText(snapshot),
        this.countBubbles(snapshot),
        ElementFactory.create("span", {
          className: `notch-chip notch-chip-${mood}`,
          text: snapshot.moodLabel,
          attributes: { title: snapshot.moodLabel },
        }),
      ],
    });
  }

  private mascotOf(snapshot: IslandSnapshot, mood: MoodName, size: number): HTMLElement {
    return ElementFactory.fromTrustedMarkup("notch-mascot", this.mascot.render(mood, snapshot.moodLabel, size));
  }

  private summaryText(snapshot: IslandSnapshot): HTMLElement {
    return ElementFactory.create("span", { className: "notch-summary", text: snapshot.closedSummary });
  }

  /** "3" for what is due and "2" for what is waiting; a zero count is left out. */
  private countBubbles(snapshot: IslandSnapshot): HTMLElement {
    const bubbles = [
      { count: snapshot.dueCount, kind: "due" },
      { count: snapshot.waitingCount, kind: "waiting" },
    ]
      .filter(({ count }) => count > 0)
      .map(({ count, kind }) =>
        ElementFactory.create("span", { className: `notch-count notch-count-${kind}`, text: String(count) }));

    return ElementFactory.create("span", {
      className: "notch-counts",
      attributes: { role: "img", "aria-label": snapshot.closedSummary, title: snapshot.closedSummary },
      children: bubbles,
    });
  }
}
