import type { IslandSnapshot, SnapshotSection, TikiAlert } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { AlertActionButtons, type AlertActionHandlers, type AlertButtonsPlace } from "./AlertActionButtons";
import type { TaskListHandlers, TaskListView } from "./TaskListView";

export interface TodayHandlers extends TaskListHandlers, AlertActionHandlers {
  /** Opens what the latest alert is about: its item, or the brief as a receipt. */
  onOpenAlert: () => void;
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
 * alert with its buttons, or the AI brief, or a button to write it); then today's
 * sections and "Delivered today".
 */
export class TodayView {
  /** The latest alert's buttons above the list, with every snooze. */
  private static readonly NOTICE_BUTTONS: AlertButtonsPlace = Object.freeze({ idPrefix: "alert", className: "settings-option", everySnooze: true });
  private readonly alertButtons: AlertActionButtons;

  constructor(
    private readonly translator: Translator,
    private readonly taskList: TaskListView,
  ) {
    this.alertButtons = new AlertActionButtons(translator);
  }

  render(data: TodayData, handlers: TodayHandlers): HTMLElement {
    const subtitle = ElementFactory.create("p", { className: "island-subtitle", text: data.snapshot.todaySubtitle });
    const list = this.taskList.render(
      TodayView.withoutFocus(data.snapshot),
      this.translator.text("island.today.empty"),
      handlers,
    );
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

  /** Today's sections without the item already shown as "Now"; a section left empty goes. */
  private static withoutFocus(snapshot: IslandSnapshot): SnapshotSection[] {
    const focusId = snapshot.focus?.item.taskId;
    return snapshot.sections
      .map((section) => ({ ...section, items: section.items.filter((item) => item.taskId !== focusId) }))
      .filter((section) => section.items.length > 0);
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
    const buttons = this.alertButtons.build(alert, handlers, TodayView.NOTICE_BUTTONS);
    return ElementFactory.create("div", {
      className: "island-notice",
      children: [content, ...(buttons.length > 0 ? [ElementFactory.create("div", { className: "island-notice-actions", children: buttons })] : [])],
    });
  }

  /** The buttons of one action: Done, one per snooze length, or Join; opening is the card itself. */


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
