import type { MeetingItem, SnapshotItem } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
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
  constructor(private readonly translator: Translator) {}

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
              : this.row(entry.reminder, handlers))),
          ]),
        }),
      ],
    });
  }

  /** Reminders with a time and every meeting, by time; all-day meetings lead their day. */
  private static entries(items: readonly SnapshotItem[], meetings: readonly MeetingItem[]): AgendaEntry[] {
    const reminders: AgendaEntry[] = items
      .filter((item) => item.urgency === "reminder" && item.at !== null)
      .map((reminder) => ({ at: reminder.at ?? "", reminder }));
    const calendar: AgendaEntry[] = meetings.map((meeting) => ({ at: meeting.startsAt, meeting }));
    return [...reminders, ...calendar].sort((left, right) => left.at.localeCompare(right.at));
  }

  private meetingRow(meeting: MeetingItem, handlers: AgendaHandlers): HTMLElement {
    const time = meeting.allDay
      ? this.translator.text("meeting.all_day")
      : this.translator.format("meeting.time_range", { start: CoreDateTimeText.time(meeting.startsAt), end: CoreDateTimeText.time(meeting.endsAt) });
    const title = ElementFactory.create("span", {
      className: "island-row-text",
      text: meeting.title || this.translator.text("meeting.untitled"),
      children: meeting.location ? [ElementFactory.create("small", { text: meeting.location })] : [],
    });
    const joinUrl = meeting.joinUrl;
    const join = joinUrl === null ? [] : [this.joinButton(joinUrl, handlers)];
    return ElementFactory.create("div", {
      className: "island-row agenda-row agenda-meeting",
      attributes: { title: meeting.accountEmail },
      children: [
        ElementFactory.create("span", { className: "agenda-time", text: time }),
        title,
        ElementFactory.fromTrustedMarkup("agenda-bell", ICONS.calendar),
        ...join,
      ],
    });
  }

  private joinButton(url: string, handlers: AgendaHandlers): HTMLElement {
    const join = ElementFactory.textButton({
      className: "agenda-join",
      text: this.translator.text("meeting.join"),
    }, () => handlers.onJoinMeeting(url));
    return join;
  }

  private row(item: SnapshotItem, handlers: TaskListHandlers): HTMLElement {
    const time = item.at === null ? "" : CoreDateTimeText.time(item.at);
    const repeat = item.repeat !== "never" ? this.translator.text(`repeat.${item.repeat}`) : null;
    const title = ElementFactory.textButton({
      className: "island-row-text",
      text: item.title,
      attributes: { title: this.translator.text("island.row.edit") },
      children: repeat ? [ElementFactory.create("small", { text: repeat })] : [],
    }, () => handlers.onOpenTask(item.taskId));
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
