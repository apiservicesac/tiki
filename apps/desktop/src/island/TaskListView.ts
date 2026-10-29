import type { DeliveredItem, Priority, SnapshotItem, SnapshotSection } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { ItemContext } from "./ItemContext";
import { ItemRow } from "./ItemRow";

export interface TaskListHandlers {
  onDeliverTask: (taskId: number) => void;
  /** Brings a delivered task back to "to do". */
  onReopenTask: (taskId: number) => void;
  onOpenTask: (taskId: number) => void;
}

/** Draws grouped task lines, each with its project color and a button to deliver it;
 * clicking a line opens it for editing. */
export class TaskListView {
  private readonly itemRow: ItemRow;

  constructor(private readonly translator: Translator) {
    this.itemRow = new ItemRow(translator);
  }

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
    return this.itemRow.render({
      attributes: { "data-task-id": String(item.taskId) },
      lead: this.itemRow.colorBar(item.projectColor),
      mark: this.priorityMark(item.priority),
      title: item.title,
      context: ItemContext.describe(item.projectName, item.tags),
      onOpen: () => handlers.onOpenTask(item.taskId),
      badge: this.itemRow.when(item.urgency, item.detailLabel),
      trailing: this.itemRow.deliverButton(() => handlers.onDeliverTask(item.taskId)),
    });
  }

  /** One delivered task: project color, title (opens it) with its project and tags, when it was delivered and a button to reopen it. */
  deliveredRow(item: DeliveredItem, handlers: TaskListHandlers): HTMLElement {
    return this.itemRow.render({
      variant: "island-row-delivered",
      attributes: { "data-task-id": String(item.taskId) },
      lead: this.itemRow.colorBar(item.projectColor),
      title: item.title,
      context: ItemContext.describe(item.projectName, item.tags),
      onOpen: () => handlers.onOpenTask(item.taskId),
      badge: this.itemRow.when("done", item.deliveredLabel),
      trailing: this.itemRow.reopenButton(() => handlers.onReopenTask(item.taskId)),
    });
  }

  /** "!!" for urgent and "!" for important work, before the title. */
  private priorityMark(priority: Priority): HTMLElement | null {
    const mark = UI_CONFIG.textMarks.priority[priority];
    return mark
      ? ElementFactory.create("span", {
        className: `island-priority island-priority-${priority}`,
        text: mark,
        attributes: { title: this.translator.text(`priority.${priority}`) },
      })
      : null;
  }
}
