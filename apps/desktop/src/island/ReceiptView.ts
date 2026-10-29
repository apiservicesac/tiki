import type { IslandSnapshot, TikiAlert } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { ItemContext } from "./ItemContext";

/**
 * The brief drawn as the receipt of the design: a header, one block per section with its
 * lines, Tiki's suggestion and the day's totals.
 */
export class ReceiptView {
  constructor(private readonly translator: Translator) {}

  render(snapshot: IslandSnapshot, alert: TikiAlert | null, dayBrief: string | null): HTMLElement {
    const blocks = snapshot.sections.length > 0
      ? snapshot.sections.map((section) =>
        ElementFactory.create("section", {
          className: "receipt-block",
          children: [
            ElementFactory.create("div", {
              className: "receipt-label",
              children: [
                ElementFactory.create("span", { text: section.title }),
                ElementFactory.create("span", { text: String(section.items.length) }),
              ],
            }),
            ...section.items.map((item) =>
              ElementFactory.create("div", {
                className: "receipt-line",
                children: [
                  ElementFactory.create("span", { text: [item.title, ItemContext.describe(item.projectName, [])].filter((part) => part !== null).join(UI_CONFIG.textMarks.separator) }),
                  ElementFactory.create("em", { className: `receipt-when-${item.urgency}`, text: item.detailLabel }),
                ],
              })),
          ],
        }))
      : [ElementFactory.create("p", { className: "receipt-clear", text: this.translator.text("receipt.all_clear") })];
    const suggestion = this.suggestion(alert, dayBrief);

    return ElementFactory.create("article", {
      className: "receipt",
      attributes: { "aria-label": alert?.title ?? this.translator.text("receipt.title") },
      children: [
        ElementFactory.create("header", {
          className: "receipt-head",
          children: [
            ElementFactory.create("strong", { text: alert?.title ?? this.translator.text("receipt.title") }),
            ElementFactory.create("span", { text: snapshot.todaySubtitle }),
          ],
        }),
        ...blocks,
        ...(suggestion
          ? [ElementFactory.create("p", {
            className: "receipt-tip",
            children: [
              ElementFactory.create("b", { text: this.translator.format("punctuation.label", { label: this.translator.text("receipt.suggestion") }) }),
              document.createTextNode(suggestion),
            ],
          })]
          : []),
        ElementFactory.create("p", {
          className: "receipt-foot",
          text: this.translator.format("receipt.totals", {
            open: snapshot.openCount,
            waiting: snapshot.waitingCount,
            delivered: snapshot.deliveredToday,
          }),
        }),
      ],
    });
  }

  /** The AI brief when there is one; otherwise what the reminder suggests (start with, follow up). */
  private suggestion(alert: TikiAlert | null, dayBrief: string | null): string | null {
    if (dayBrief) {
      return dayBrief;
    }
    const tips = alert?.body.split("\n").slice(1) ?? [];
    return tips.length > 0 ? tips.join(" ") : null;
  }
}
