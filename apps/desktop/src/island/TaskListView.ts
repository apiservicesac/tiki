import type { DeliveredItem, Priority, SnapshotItem, SnapshotSection } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";

export interface TaskListHandlers {
  onDeliverTask: (taskId: number) => void;
  onOpenTask: (taskId: number) => void;
}

/** Draws grouped task lines, each with its project color and a button to deliver it;
 * clicking a line opens it for editing. */
export class TaskListView {
  constructor(private readonly translator: Translator) {}

  render(sections: SnapshotSection[], emptyMessage: string, handlers: TaskListHandlers): HTMLElement {
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
        ...section.items.map((item) => this.row(item, handlers)),
      ]),
    });
  }

  /** One open task: project color, title (opens it), when it is due, and a deliver button. */
  row(item: SnapshotItem, handlers: TaskListHandlers): HTMLElement {
    const colorBar = ElementFactory.create("span", { className: "island-row-bar" });
    colorBar.style.background = item.projectColor ?? "var(--island-faint)";

    const deliverButton = ElementFactory.iconButton(
      ICONS.check,
      this.translator.text("island.row.done"),
      () => handlers.onDeliverTask(item.taskId),
    );
    deliverButton.classList.add("island-row-done");

    const text = ElementFactory.create("button", {
      className: "island-row-text",
      attributes: { type: "button", title: this.translator.text("island.row.edit") },
      children: [
        ...this.priorityMark(item.priority),
        document.createTextNode(item.title),
        ...TaskListView.context(item.projectName, item.tags),
      ],
    });
    text.addEventListener("click", (event) => {
      event.stopPropagation();
      handlers.onOpenTask(item.taskId);
    });

    return ElementFactory.create("div", {
      className: "island-row",
      attributes: { "data-task-id": String(item.taskId) },
      children: [
        colorBar,
        text,
        ElementFactory.create("span", {
          className: `island-when island-when-${item.urgency}`,
          text: item.detailLabel,
        }),
        deliverButton,
      ],
    });
  }

  /** One delivered task: project color, title (opens it, to reopen it) and when it was delivered. */
  deliveredRow(item: DeliveredItem, onOpen: (taskId: number) => void): HTMLElement {
    const colorBar = ElementFactory.create("span", { className: "island-row-bar" });
    colorBar.style.background = item.projectColor ?? "var(--island-faint)";
    const text = ElementFactory.create("button", {
      className: "island-row-text",
      text: item.title,
      attributes: { type: "button", title: this.translator.text("island.row.edit") },
      children: item.projectName ? [ElementFactory.create("small", { text: item.projectName })] : [],
    });
    text.addEventListener("click", (event) => {
      event.stopPropagation();
      onOpen(item.taskId);
    });
    return ElementFactory.create("div", {
      className: "island-row island-row-delivered",
      attributes: { "data-task-id": String(item.taskId) },
      children: [
        colorBar,
        text,
        ElementFactory.create("span", { className: "island-when island-when-done", text: item.deliveredLabel }),
        ElementFactory.fromTrustedMarkup("island-row-check", ICONS.check),
      ],
    });
  }

  /** "Altamira · #billing" under the title; nothing when the item has neither. */
  private static context(projectName: string | null, tags: readonly string[]): HTMLElement[] {
    const parts = [...(projectName ? [projectName] : []), ...tags.map((tag) => `#${tag}`)];
    return parts.length > 0 ? [ElementFactory.create("small", { text: parts.join(UI_CONFIG.textMarks.separator) })] : [];
  }

  /** "!!" for urgent and "!" for important work, before the title. */
  private priorityMark(priority: Priority): HTMLElement[] {
    const mark = UI_CONFIG.textMarks.priority[priority];
    return mark
      ? [ElementFactory.create("span", {
        className: `island-priority island-priority-${priority}`,
        text: mark,
        attributes: { title: this.translator.text(`priority.${priority}`) },
      })]
      : [];
  }
}
