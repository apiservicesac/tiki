import type { MeetingItem, SnapshotItem } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import { ItemRow } from "./ItemRow";
import type { TaskListHandlers } from "./TaskListView";

export interface AgendaHandlers extends TaskListHandlers {
  onJoinMeeting: (url: string) => void;
}

/** One line of the agenda: a reminder of the user's or a meeting from a calendar. */
type AgendaEntry = { at: string; reminder: SnapshotItem } | { at: string; meeting: MeetingItem };

/**
 * Agenda: everything that happens at a time, by day. Reminders and calendar meetings share
 * one list ordered by time. A reminder opens on click and has a check; a meeting shows its
 * time range and place, and a Join button when it has a video call.
 */
export class AgendaView {
  private readonly itemRow: ItemRow;

  constructor(private readonly translator: Translator) {
    this.itemRow = new ItemRow(translator);
  }

  render(
    items: readonly SnapshotItem[],
    meetings: readonly MeetingItem[],
    today: Date,
    handlers: AgendaHandlers,
  ): HTMLElement {
    const entries = AgendaView.entries(items, meetings);
    if (entries.length === 0) {
      return ElementFactory.create("div", {
        className: "island-tab-content",
        children: [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("agenda.empty") })],
      });
    }
    const days = new Map<string, AgendaEntry[]>();
    entries.forEach((entry) => {
      const day = CoreDateTimeText.day(entry.at);
      days.set(day, [...(days.get(day) ?? []), entry]);
    });
    return ElementFactory.create("div", {
      className: "island-tab-content",
      children: [
        ElementFactory.create("div", {
          className: "island-list agenda-list",
          children: [...days.entries()].flatMap(([day, dayEntries]) => [
            ElementFactory.create("div", { className: "island-group", text: this.dayTitle(day, today) }),
            ...dayEntries.map((entry) => ("meeting" in entry
              ? this.meetingRow(entry.meeting, handlers)
              : this.reminderRow(entry.reminder, entry.at, handlers))),
          ]),
        }),
      ],
    });
  }

  /** Reminders with a time and every meeting, by time; all-day meetings lead their day. */
  private static entries(items: readonly SnapshotItem[], meetings: readonly MeetingItem[]): AgendaEntry[] {
    const reminders: AgendaEntry[] = items
      .filter((item) => item.kind === "reminder" && item.at !== null)
      .map((reminder) => ({ at: reminder.at ?? "", reminder }));
    const calendar: AgendaEntry[] = meetings.map((meeting) => ({ at: meeting.startsAt, meeting }));
    return [...reminders, ...calendar].sort((left, right) => left.at.localeCompare(right.at));
  }

  /** A meeting: its time range, title and place, and Join when it has a video call. */
  private meetingRow(meeting: MeetingItem, handlers: AgendaHandlers): HTMLElement {
    const time = meeting.allDay
      ? this.translator.text("meeting.all_day")
      : this.translator.format("meeting.time_range", { start: CoreDateTimeText.time(meeting.startsAt), end: CoreDateTimeText.time(meeting.endsAt) });
    const joinUrl = meeting.joinUrl;
    return this.itemRow.render({
      variant: "agenda-row agenda-meeting",
      attributes: { title: meeting.accountEmail },
      lead: this.itemRow.time(time),
      title: meeting.title || this.translator.text("meeting.untitled"),
      context: meeting.location || null,
      badge: this.itemRow.kindIcon(ICONS.calendar),
      trailing: joinUrl === null
        ? null
        : ElementFactory.textButton({ className: "agenda-join", text: this.translator.text("meeting.join") }, () => handlers.onJoinMeeting(joinUrl)),
    });
  }

  /** A reminder: its time, title (opens it) and repeat, and a check to deliver it. */
  private reminderRow(reminder: SnapshotItem, at: string, handlers: TaskListHandlers): HTMLElement {
    return this.itemRow.render({
      variant: "agenda-row",
      attributes: { "data-task-id": String(reminder.taskId) },
      lead: this.itemRow.time(CoreDateTimeText.time(at)),
      title: reminder.title,
      context: reminder.repeat === "never" ? null : this.translator.text(`repeat.${reminder.repeat}`),
      onOpen: () => handlers.onOpenTask(reminder.taskId),
      badge: this.itemRow.kindIcon(ICONS.bell),
      trailing: this.itemRow.deliverButton(() => handlers.onDeliverTask(reminder.taskId)),
    });
  }

  /** "Hoy", "Mañana" or the long date. */
  private dayTitle(isoDay: string, today: Date): string {
    const days = CoreDateTimeText.daysFrom(today, isoDay);
    if (days === 0) {
      return this.translator.text("agenda.today");
    }
    if (days === 1) {
      return this.translator.text("agenda.tomorrow");
    }
    return new Intl.DateTimeFormat(this.translator.localeCode, { weekday: "long", day: "numeric", month: "long" }).format(CoreDateTimeText.startOf(isoDay));
  }
}
