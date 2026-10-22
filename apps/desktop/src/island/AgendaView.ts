import type { SnapshotItem } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import type { TaskListHandlers } from "./TaskListView";

/**
 * Agenda: everything that happens at a time, by day. Today it holds the user's reminders;
 * meetings from the calendar join here later. Each line shows the time and how often it
 * repeats; clicking it opens the reminder, the check marks it done.
 */
export class AgendaView {
  constructor(private readonly translator: Translator) {}

  render(items: readonly SnapshotItem[], today: Date, handlers: TaskListHandlers): HTMLElement {
    const reminders = items.filter((item) => item.urgency === "reminder" && item.at !== null);
    if (reminders.length === 0) {
      return ElementFactory.create("div", {
        className: "island-tab-content",
        children: [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("agenda.empty") })],
      });
    }
    const days = new Map<string, SnapshotItem[]>();
    reminders.forEach((item) => {
      const day = (item.at ?? "").slice(0, 10);
      days.set(day, [...(days.get(day) ?? []), item]);
    });
    return ElementFactory.create("div", {
      className: "island-tab-content",
      children: [
        ElementFactory.create("div", {
          className: "island-list agenda-list",
          children: [...days.entries()].flatMap(([day, dayItems]) => [
            ElementFactory.create("div", { className: "island-group", text: this.dayTitle(day, today) }),
            ...dayItems.map((item) => this.row(item, handlers)),
          ]),
        }),
      ],
    });
  }

  private row(item: SnapshotItem, handlers: TaskListHandlers): HTMLElement {
    const time = (item.at ?? "").slice(11, 16);
    const repeat = item.repeat !== "never" ? this.translator.text(`repeat.${item.repeat}`) : null;
    const title = ElementFactory.create("button", {
      className: "island-row-text",
      text: item.title,
      attributes: { type: "button", title: this.translator.text("island.row.edit") },
      children: repeat ? [ElementFactory.create("small", { text: repeat })] : [],
    });
    title.addEventListener("click", (event) => {
      event.stopPropagation();
      handlers.onOpenTask(item.taskId);
    });
    const done = ElementFactory.iconButton(ICONS.check, this.translator.text("island.row.done"), () => handlers.onDeliverTask(item.taskId));
    done.classList.add("island-row-done");
    return ElementFactory.create("div", {
      className: "island-row agenda-row",
      attributes: { "data-task-id": String(item.taskId) },
      children: [
        ElementFactory.create("span", { className: "agenda-time", text: time }),
        title,
        ElementFactory.fromTrustedMarkup("agenda-bell", ICONS.bell),
        done,
      ],
    });
  }

  /** "Hoy", "Mañana" or the long date. */
  private dayTitle(isoDay: string, today: Date): string {
    const [year, month, day] = isoDay.split("-").map(Number) as [number, number, number];
    const date = new Date(year, month - 1, day);
    const startOfToday = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    const days = Math.round((date.getTime() - startOfToday.getTime()) / 86_400_000);
    if (days === 0) {
      return this.translator.text("agenda.today");
    }
    if (days === 1) {
      return this.translator.text("agenda.tomorrow");
    }
    return new Intl.DateTimeFormat(this.translator.localeCode, { weekday: "long", day: "numeric", month: "long" }).format(date);
  }
}
