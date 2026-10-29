import type { AlertAction, IslandSnapshot, TikiAlert } from "../backend/contracts";
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
   * The alert's title, summary and tips, opening what it is about when the core offers that,
   * and a button for each other action it offers.
   */
  private alertCard(alert: TikiAlert, handlers: TodayHandlers): HTMLElement {
    const text = [
      ElementFactory.create("strong", { text: alert.title }),
      ...[alert.summary, ...alert.tips].map((line) => ElementFactory.create("p", { text: line })),
    ];
    const content = alert.actions.includes("open")
      ? ElementFactory.textButton({
        className: "island-notice-open",
        attributes: { title: this.translator.text("island.alert.open") },
        children: text,
      }, handlers.onOpenAlert)
      : ElementFactory.create("div", { className: "island-notice-open", children: text });
    const buttons = alert.actions.flatMap((action) => this.actionButtons(action, alert, handlers));
    return ElementFactory.create("div", {
      className: "island-notice",
      children: [content, ...(buttons.length > 0 ? [ElementFactory.create("div", { className: "island-notice-actions", children: buttons })] : [])],
    });
  }

  /** The buttons of one action: Done, one per snooze length, or Join; opening is the card itself. */
  private actionButtons(action: AlertAction, alert: TikiAlert, handlers: TodayHandlers): HTMLElement[] {
    const { taskId, link } = alert;
    switch (action) {
      case "open":
        return [];
      case "done":
        return [this.alertOption("alert.done", "alert-done", handlers.onDismissAlert)];
      case "snooze":
        return taskId === null
          ? []
          : UI_CONFIG.snoozeMinutes.map(({ minutes, labelKey }) =>
            this.alertOption(labelKey, `alert-snooze-${minutes}`, () => handlers.onSnooze(taskId, minutes)));
      case "join":
        return link === null ? [] : [this.alertOption("meeting.join", "alert-join", () => handlers.onJoinMeeting(link))];
    }
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
