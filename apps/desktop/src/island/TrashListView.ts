import type { TrashedItem } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";
import type { Translator } from "../i18n/Translator";
import { ItemContext } from "./ItemContext";
import { ItemRow } from "./ItemRow";

export interface TrashHandlers {
  onRestoreTask: (taskId: number) => void;
  onDeleteForever: (taskId: number) => void;
  onEmptyTrash: () => void;
}

/** What waits for a second click before it happens: one item, or the whole trash. */
type PendingDeletion = { taskId: number } | "all" | null;

/**
 * The trash, in the Tasks tab: how long things stay there, a button to empty it, and each
 * item with when it went there, when it goes away, Restore and Delete forever. Deleting asks
 * for a second click, since it cannot be undone.
 */
export class TrashListView {
  private readonly itemRow: ItemRow;
  private pending: PendingDeletion = null;

  constructor(private readonly translator: Translator) {
    this.itemRow = new ItemRow(translator);
  }

  /** Forgets a deletion waiting for its second click, for example when the view changes. */
  reset(): void {
    this.pending = null;
  }

  render(items: readonly TrashedItem[], query: string, retentionDays: number, handlers: TrashHandlers, redraw: () => void): HTMLElement[] {
    const shown = items.filter((item) =>
      TextMatcher.matches(item.title, query) || (item.projectName !== null && TextMatcher.matches(item.projectName, query)));
    if (shown.length === 0) {
      return [];
    }
    return [this.header(retentionDays, handlers, redraw), ...shown.map((item) => this.row(item, handlers, redraw))];
  }

  private header(retentionDays: number, handlers: TrashHandlers, redraw: () => void): HTMLElement {
    const confirming = this.pending === "all";
    const empty = ElementFactory.textButton({
      className: confirming ? "trash-empty trash-confirm" : "trash-empty",
      text: this.translator.text(confirming ? "trash.empty_confirm" : "trash.empty_action"),
      attributes: { id: "trash-empty" },
    }, () => {
      this.pending = confirming ? null : "all";
      if (confirming) {
        handlers.onEmptyTrash();
      }
      redraw();
    });
    return ElementFactory.create("div", {
      className: "trash-header",
      children: [
        ElementFactory.create("p", { className: "trash-note", text: this.translator.format("trash.retention_note", { days: retentionDays }) }),
        empty,
      ],
    });
  }

  private row(item: TrashedItem, handlers: TrashHandlers, redraw: () => void): HTMLElement {
    const confirming = typeof this.pending === "object" && this.pending?.taskId === item.taskId;
    const restore = ElementFactory.textButton({
      className: "island-row-reopen",
      text: this.translator.text("trash.restore"),
    }, () => handlers.onRestoreTask(item.taskId));
    const remove = confirming
      ? ElementFactory.textButton({ className: "trash-confirm", text: this.translator.text("trash.delete_forever_confirm") }, () => {
        this.pending = null;
        handlers.onDeleteForever(item.taskId);
      })
      : ElementFactory.iconButton(ICONS.trash, this.translator.text("trash.delete_forever"), () => {
        this.pending = { taskId: item.taskId };
        redraw();
      });
    return this.itemRow.render({
      variant: "island-row-delivered",
      attributes: { "data-task-id": String(item.taskId) },
      lead: this.itemRow.colorBar(item.projectColor),
      title: item.title,
      context: ItemContext.join([item.projectName, item.deletesInLabel]),
      badge: this.itemRow.when("normal", item.trashedLabel),
      trailing: ElementFactory.create("span", { className: "trash-actions", children: [restore, remove] }),
    });
  }
}
