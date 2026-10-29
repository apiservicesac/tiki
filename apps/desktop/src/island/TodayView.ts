import type { IslandSnapshot, TikiAlert } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { TaskListHandlers, TaskListView } from "./TaskListView";

export interface TodayHandlers extends TaskListHandlers {
  /** Opens what the latest alert is about: its item, or the brief as a receipt. */
  onOpenAlert: () => void;
  onDismissAlert: () => void;
  /** Puts off the reminder of the latest alert. */
  onSnooze: (taskId: number, minutes: number) => void;
  /** Opens a meeting's video call. */
  onJoinMeeting: (url: string) => void;
  onPrepareBrief: () => void;
  /** "Delivered today" at the end: opens Tasks › Delivered. */
  onShowDelivered: () => void;
}

export interface TodayData {
  snapshot: IslandSnapshot;
  alert: TikiAlert | null;
  dayBrief: string | null;
  preparingBrief: boolean;
  aiReady: boolean;
}

/**
 * Today: only what matters today. On top, "Now: do X" and one summary card (the latest
 * reminder with its buttons, or the AI brief, or a button to write it); then today's
 * sections and "Delivered today".
 */
export class TodayView {
  constructor(
    private readonly translator: Translator,
    private readonly taskList: TaskListView,
  ) {}

  render(data: TodayData, handlers: TodayHandlers): HTMLElement {
    const subtitle = ElementFactory.create("p", { className: "island-subtitle", text: data.snapshot.todaySubtitle });
    const list = this.taskList.render(data.snapshot.sections, this.translator.text("island.today.empty"), handlers);
    return ElementFactory.create("div", {
      className: "island-tab-content",
      children: [
        subtitle,
        ...this.focusCard(data.snapshot, handlers),
        ...this.summaryCard(data, handlers),
        list,
        ...this.deliveredToday(data.snapshot, handlers),
      ],
    });
  }

  /** "Now · Due today: Send the report", opening the item. */
  private focusCard(snapshot: IslandSnapshot, handlers: TodayHandlers): HTMLElement[] {
    const focus = snapshot.focus;
    if (!focus) {
      return [];
    }
    return [ElementFactory.textButton({
      className: "island-focus",
      attributes: { id: "today-focus", "data-task-id": String(focus.item.taskId) },
      children: [
        ElementFactory.create("span", { className: "island-focus-label", text: this.translator.format("focus.label", { reason: focus.reason }) }),
        ElementFactory.create("strong", { text: focus.item.title }),
        ...(focus.item.projectName ? [ElementFactory.create("small", { text: focus.item.projectName })] : []),
      ],
    }, () => handlers.onOpenTask(focus.item.taskId))];
  }

  /** One card: the latest alert, else the AI brief, else (with AI) a button to write it. */
  private summaryCard(data: TodayData, handlers: TodayHandlers): HTMLElement[] {
    if (data.alert) {
      return [this.alertCard(data.alert, handlers)];
    }
    if (data.dayBrief || data.preparingBrief) {
      return [ElementFactory.textButton({
        className: data.preparingBrief ? "island-summary island-summary-preparing" : "island-summary",
        text: data.preparingBrief ? this.translator.text("summary.preparing") : data.dayBrief ?? "",
        attributes: { id: "today-summary" },
      }, handlers.onOpenAlert)];
    }
    if (!data.aiReady) {
      return [];
    }
    return [ElementFactory.textButton({
      className: "island-summary-prepare",
      text: this.translator.text("summary.prepare"),
      attributes: { id: "prepare-brief" },
    }, handlers.onPrepareBrief)];
  }

  /**
   * A reminder offers Done and Snooze, a meeting with a video call offers Join; a brief or a
   * task reminder opens what it is about.
   */
  private alertCard(alert: TikiAlert, handlers: TodayHandlers): HTMLElement {
    const open = ElementFactory.textButton({
      className: "island-notice-open",
      attributes: { title: this.translator.text("island.alert.open") },
      children: [
        ElementFactory.create("strong", { text: alert.title }),
        ElementFactory.create("p", { text: alert.body }),
      ],
    }, handlers.onOpenAlert);
    const buttons = this.alertButtons(alert, handlers);
    return ElementFactory.create("div", {
      className: "island-notice",
      children: [open, ...(buttons.length > 0 ? [ElementFactory.create("div", { className: "island-notice-actions", children: buttons })] : [])],
    });
  }

  private alertButtons(alert: TikiAlert, handlers: TodayHandlers): HTMLElement[] {
    const { taskId, link } = alert;
    if (alert.kind === "reminder" && taskId !== null) {
      return [
        this.alertOption("alert.done", "alert-done", handlers.onDismissAlert),
        ...UI_CONFIG.snoozeMinutes.map(({ minutes, labelKey }) =>
          this.alertOption(labelKey, `alert-snooze-${minutes}`, () => handlers.onSnooze(taskId, minutes))),
      ];
    }
    if (alert.kind === "meeting" && link !== null) {
      return [this.alertOption("meeting.join", "alert-join", () => handlers.onJoinMeeting(link))];
    }
    return [];
  }

  /** One of the alert's buttons: done, a snooze or join. */
  private alertOption(labelKey: string, id: string, onPress: () => void): HTMLElement {
    return ElementFactory.textButton({
      className: "settings-option",
      text: this.translator.text(labelKey),
      attributes: { id },
    }, onPress);
  }

  /** "Delivered today (3)", leading to the history; nothing on a day with no deliveries. */
  private deliveredToday(snapshot: IslandSnapshot, handlers: TodayHandlers): HTMLElement[] {
    if (snapshot.deliveredToday === 0) {
      return [];
    }
    return [ElementFactory.textButton({
      className: "island-delivered-link",
      text: this.translator.format("island.delivered_today", { count: snapshot.deliveredToday }),
      attributes: { id: "delivered-today" },
    }, handlers.onShowDelivered)];
  }
}
