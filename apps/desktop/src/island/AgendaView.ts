import type { MeetingItem, SnapshotItem } from "../backend/contracts";
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
      const day = entry.at.slice(0, 10);
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
      : this.translator.format("meeting.time_range", { start: meeting.startsAt.slice(11, 16), end: meeting.endsAt.slice(11, 16) });
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
    const join = ElementFactory.create("button", {
      className: "agenda-join",
      text: this.translator.text("meeting.join"),
      attributes: { type: "button" },
    });
    join.addEventListener("click", (event) => {
      event.stopPropagation();
      handlers.onJoinMeeting(url);
    });
    return join;
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
