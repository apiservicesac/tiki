import type { SnapshotItem, SnapshotSection } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";

/** Draws grouped task lines, each with its client color and a button to deliver it. */
export class TaskListView {
  constructor(private readonly translator: Translator) {}

  render(sections: SnapshotSection[], emptyMessage: string, onDeliver: (taskId: number) => void): HTMLElement {
    if (sections.length === 0) {
      return ElementFactory.create("p", { className: "island-empty", text: emptyMessage });
    }
    return ElementFactory.create("div", {
      className: "island-list",
      children: sections.flatMap((section) => [
        ElementFactory.create("div", {
          className: "island-group",
          text: section.title,
          attributes: { "data-section": section.key },
        }),
        ...section.items.map((item) => this.row(item, onDeliver)),
      ]),
    });
  }

  private row(item: SnapshotItem, onDeliver: (taskId: number) => void): HTMLElement {
    const colorBar = ElementFactory.create("span", { className: "island-row-bar" });
    colorBar.style.background = item.clientColor ?? "var(--island-faint)";

    const deliverButton = ElementFactory.iconButton(
      ICONS.check,
      this.translator.text("island.row.done"),
      () => onDeliver(item.taskId),
    );
    deliverButton.classList.add("island-row-done");

    return ElementFactory.create("div", {
      className: "island-row",
      attributes: { "data-task-id": String(item.taskId) },
      children: [
        colorBar,
        ElementFactory.create("span", {
          className: "island-row-text",
          text: item.title,
          children: item.clientName ? [ElementFactory.create("small", { text: item.clientName })] : [],
        }),
        ElementFactory.create("span", {
          className: `island-when island-when-${item.urgency}`,
          text: item.detailLabel,
        }),
        deliverButton,
      ],
    });
  }
}
