import type { AgendaDay, AgendaEntry, MeetingItem, SnapshotItem } from "../backend/contracts";
import { AccountBadge } from "../dom/AccountBadge";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import { ItemContext } from "./ItemContext";
import { ItemRow } from "./ItemRow";
import type { TaskListHandlers } from "./TaskListView";

export interface AgendaHandlers extends TaskListHandlers {
  onJoinMeeting: (url: string) => void;
}

/**
 * Agenda: everything that happens at a time, by day, as the core lays it out: calendar
 * meetings, reminders and the alarms set on tasks, each day under its title and each line
 * with its time. A reminder or a task opens on click and has a check; a meeting shows its
 * place, its account's badge, and a Join button when it has a video call.
 */
export class AgendaView {
  private readonly itemRow: ItemRow;

  constructor(private readonly translator: Translator) {
    this.itemRow = new ItemRow(translator);
  }

  render(days: readonly AgendaDay[], handlers: AgendaHandlers): HTMLElement {
    if (days.length === 0) {
      return ElementFactory.create("div", {
        className: "island-tab-content",
        children: [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("agenda.empty") })],
      });
    }
    return ElementFactory.create("div", {
      className: "island-tab-content",
      children: [
        ElementFactory.create("div", {
          className: "island-list agenda-list",
          children: days.flatMap((day) => [
            ElementFactory.create("div", { className: "island-group", text: day.label }),
            ...day.entries.map((entry) => this.row(entry, handlers)),
          ]),
        }),
      ],
    });
  }

  private row(entry: AgendaEntry, handlers: AgendaHandlers): HTMLElement {
    switch (entry.kind) {
      case "meeting":
        return this.meetingRow(entry.meeting, entry.timeLabel, handlers);
      case "reminder":
        return this.timedItemRow(entry.item, entry.timeLabel, this.repeatText(entry.item), handlers);
      case "task":
        return this.timedItemRow(entry.item, entry.timeLabel, ItemContext.describe(entry.item.projectName, entry.item.tags), handlers);
    }
  }

  /** A meeting: its time range, title, place and account's badge, and Join when it has a video call. */
  private meetingRow(meeting: MeetingItem, timeLabel: string, handlers: AgendaHandlers): HTMLElement {
    const joinUrl = meeting.joinUrl;
    return this.itemRow.render({
      variant: "agenda-row agenda-meeting",
      attributes: { title: meeting.account.email },
      lead: this.itemRow.time(timeLabel),
      title: meeting.title || this.translator.text("meeting.untitled"),
      context: meeting.location || null,
      badge: AccountBadge.render(meeting.account, meeting.platform ?? meeting.account.provider),
      trailing: joinUrl === null
        ? null
        : ElementFactory.textButton({ className: "agenda-join", text: this.translator.text("meeting.join") }, () => handlers.onJoinMeeting(joinUrl)),
    });
  }

  /** A reminder or a task with an alarm: its time, title (opens it) and context, and a check to deliver it. */
  private timedItemRow(item: SnapshotItem, timeLabel: string, context: string | null, handlers: TaskListHandlers): HTMLElement {
    return this.itemRow.render({
      variant: "agenda-row",
      attributes: { "data-task-id": String(item.taskId) },
      lead: this.itemRow.time(timeLabel),
      title: item.title,
      context,
      onOpen: () => handlers.onOpenTask(item.taskId),
      badge: this.itemRow.kindIcon(ICONS.bell),
      trailing: this.itemRow.deliverButton(() => handlers.onDeliverTask(item.taskId)),
    });
  }

  /** How often a reminder comes back; nothing for a one-time reminder. */
  private repeatText(reminder: SnapshotItem): string | null {
    return reminder.repeat === "never" ? null : this.translator.text(`repeat.${reminder.repeat}`);
  }
}
