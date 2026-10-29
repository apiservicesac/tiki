import type { MoodName, UserPreferences } from "../backend/contracts";
import type { CaptureView, CaptureViewHandlers } from "../capture/CaptureView";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { MascotRenderer } from "../mascot/MascotRenderer";
import { HeaderView, type HeaderHandlers, type IslandTabLabel } from "./HeaderView";
import type { IslandMode, IslandState, IslandTab } from "./islandState";
import { NotchView } from "./NotchView";
import { SettingsView, type SettingsHandlers } from "./SettingsView";
import { TaskEditView, type TaskEditHandlers } from "./TaskEditView";
import { ProjectsPanelView, type ProjectsPanelHandlers } from "./ProjectsPanelView";
import { ReceiptView } from "./ReceiptView";
import { TaskListView } from "./TaskListView";
import { TasksTabView, type TasksTabHandlers } from "./TasksTabView";
import { AgendaView } from "./AgendaView";
import { TodayView, type TodayHandlers } from "./TodayView";
import type { HitArea, TikiAlert } from "../backend/contracts";
import { ContentTransition, type ContentSize } from "./ContentTransition";
import { FocusKeeper } from "../dom/FocusKeeper";

export interface IslandViewHandlers
  extends HeaderHandlers, SettingsHandlers, TasksTabHandlers, ProjectsPanelHandlers, TodayHandlers {
  onExpand: () => void;
  /** The island changed size without a redraw: the click-through area must follow. */
  onContentResize: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
  capture: CaptureViewHandlers;
  edit: TaskEditHandlers;
}

/** One tab of the expanded island: its label and what it draws. */
interface IslandTabEntry {
  labelKey: string;
  render: (state: IslandState, handlers: IslandViewHandlers) => HTMLElement;
}

/**
 * Draws the island for a given state. The notch element lives for the whole session and
 * only its content changes, so its size and corners animate between states like dooai.
 * The window never resizes: the notch animates inside it with CSS only.
 */
export class IslandView {
  private readonly notchView: NotchView;
  private readonly header: HeaderView;
  private readonly taskList: TaskListView;
  private readonly settings: SettingsView;
  private readonly taskEdit: TaskEditView;
  private readonly tasksTab: TasksTabView;
  private readonly today: TodayView;
  private readonly agenda: AgendaView;
  private readonly projectsPanel: ProjectsPanelView;
  private readonly receipt: ReceiptView;
  /** The tabs in display order; a new tab (Tickets) is one more entry. */
  private readonly tabs: Readonly<Record<IslandTab, IslandTabEntry>>;
  private readonly tabLabels: readonly IslandTabLabel[];
  /** The views kept between frames are rebuilt when the language changes. */
  private shownLocale = "";
  private readonly notch: HTMLElement;
  private readonly clip: HTMLElement;
  private readonly contentTransition: ContentTransition;
  private handlers: IslandViewHandlers | null = null;
  private notchSize: ContentSize = { width: 0, height: 0 };
  private shownMode: IslandMode | null = null;
  private shownPanel = "";
  /** The alert drawn in the notch, kept until another one comes or the notch changes shape. */
  private shownAlert: TikiAlert | null = null;
  private assistantReady = false;

  constructor(
    private readonly root: HTMLElement,
    private readonly translator: Translator,
    private readonly mascot: MascotRenderer,
    private readonly captureView: CaptureView,
  ) {
    this.notchView = new NotchView(mascot, translator);
    this.header = new HeaderView(translator);
    this.taskList = new TaskListView(translator);
    this.settings = new SettingsView(translator);
    this.taskEdit = new TaskEditView(translator);
    this.tasksTab = new TasksTabView(translator, this.taskList);
    this.today = new TodayView(translator, this.taskList);
    this.agenda = new AgendaView(translator);
    this.projectsPanel = new ProjectsPanelView(translator);
    this.receipt = new ReceiptView(translator);
    this.tabs = Object.freeze({
      today: {
        labelKey: "island.tab.today",
        render: (state, handlers) => this.today.render({
          snapshot: state.snapshot,
          alert: state.alert,
          dayBrief: state.dayBrief,
          preparingBrief: state.preparingBrief,
          aiReady: this.assistantReady,
        }, handlers),
      },
      all_tasks: {
        labelKey: "island.tab.all_tasks",
        render: (state, handlers) => this.tasksTab.render(
          { items: state.snapshot.openItems, delivered: state.delivered, projects: state.projects, view: state.tasksView },
          handlers,
        ),
      },
      agenda: {
        labelKey: "island.tab.agenda",
        render: (state, handlers) => this.agenda.render(state.snapshot.agenda, handlers),
      },
    });
    // Object keys keep their order; they are exactly the tabs, as the record's type requires.
    this.tabLabels = (Object.keys(this.tabs) as IslandTab[]).map((tab) => ({ tab, labelKey: this.tabs[tab].labelKey }));
    this.clip = ElementFactory.create("div", { className: "island-clip" });
    this.notch = this.createNotch();
    this.contentTransition = new ContentTransition(this.clip, (size) => this.followContent(size));
    root.style.setProperty("--island-open-duration", `${UI_CONFIG.motion.openMs}ms`);
    root.style.setProperty("--island-close-duration", `${UI_CONFIG.motion.closeMs}ms`);
    root.style.setProperty("--island-content-out-duration", `${UI_CONFIG.motion.contentOutMs}ms`);
    root.style.setProperty("--alert-duration", `${UI_CONFIG.alertDurationMs}ms`);
    root.replaceChildren(this.notch);
  }

  render(state: IslandState, preferences: UserPreferences, handlers: IslandViewHandlers, assistantReady: boolean): void {
    this.assistantReady = assistantReady;
    this.handlers = handlers;
    const mood = IslandView.moodOf(state);
    this.root.dataset.placement = state.placement;
    this.root.dataset.orientation = IslandView.isVertical(state) ? "vertical" : "horizontal";
    this.root.ownerDocument.body.dataset.placement = state.placement;
    this.root.ownerDocument.documentElement.lang = this.translator.localeCode;
    this.notch.dataset.mode = state.mode;
    this.notch.dataset.mood = mood;
    // During a call the notch glows faintly: Tiki is there, keeping quiet.
    if (state.snapshot.inCall) {
      this.notch.dataset.activity = "in-call";
    } else {
      delete this.notch.dataset.activity;
    }
    // The root carries the mood too: the alert aura behind the notch takes its color.
    this.root.dataset.mood = mood;

    // An alert already in the notch is left as it is: redrawing it would restart its
    // countdown (the brief, a sync or a resize can redraw while it shows).
    if (this.isShowingAlert(state)) {
      return;
    }
    this.shownAlert = state.mode === "alert" ? state.alert : null;

    // Content cross-fades only when the island changes shape (hidden, compact, expanded).
    // Tabs and panels swap instantly, so switching them never flashes an empty card.
    const animate = this.shownMode !== null && state.mode !== this.shownMode;
    const panelKey = `${state.mode}:${state.panel}`;
    if (panelKey !== this.shownPanel) {
      this.captureView.reset();
      this.taskEdit.reset();
      this.projectsPanel.reset();
    }
    if (this.translator.localeCode !== this.shownLocale) {
      this.tasksTab.reset();
      this.shownLocale = this.translator.localeCode;
    }
    this.shownMode = state.mode;
    this.shownPanel = panelKey;

    const focus = FocusKeeper.remember(this.root.ownerDocument);
    const content = this.content(state, mood, preferences, handlers);
    this.resizeNotch(this.contentTransition.swap(content, animate));
    focus.restore();
  }

  private isShowingAlert(state: IslandState): boolean {
    return state.mode === "alert"
      && this.shownMode === "alert"
      && state.alert !== null
      && state.alert === this.shownAlert
      && this.translator.localeCode === this.shownLocale;
  }

  /** Where the island is inside the window, so the shell can let other clicks through. */
  hitArea(): HitArea {
    const style = getComputedStyle(this.root);
    const padding = (side: "Left" | "Right" | "Top" | "Bottom") => parseFloat(style[`padding${side}`]) || 0;
    const width = this.notchSize.width + padding("Left") + padding("Right");
    const height = this.notchSize.height + padding("Top") + padding("Bottom");
    const windowWidth = window.innerWidth;
    const windowHeight = window.innerHeight;
    const centeredX = (windowWidth - width) / 2;
    const centeredY = (windowHeight - height) / 2;
    switch (this.root.dataset.placement) {
      case "bottom":
        return { x: centeredX, y: windowHeight - height, width, height };
      case "left":
        return { x: 0, y: centeredY, width, height };
      case "right":
        return { x: windowWidth - width, y: centeredY, width, height };
      default:
        return { x: centeredX, y: 0, width, height };
    }
  }

  /** Whether the user is typing in the island (a search box, a name), so it must not fold. */
  isTyping(): boolean {
    const focused = this.root.ownerDocument.activeElement;
    return focused instanceof HTMLElement
      && this.notch.contains(focused)
      && (focused instanceof HTMLInputElement || focused instanceof HTMLTextAreaElement);
  }

  stop(): void {
    this.contentTransition.stop();
  }

  /**
   * The face Tiki makes: the alert's own while one is announced in the notch, then a short
   * reaction (happy after capturing), then the mood of the day.
   */
  private static moodOf(state: IslandState): MoodName {
    if (state.mode === "alert" && state.alert) {
      return UI_CONFIG.alertStyles[state.alert.kind].mood;
    }
    return state.reactionMood ?? state.snapshot.mood;
  }

  /** On the left and right edges the island is a column: everything stacks vertically. */
  private static isVertical(state: IslandState): boolean {
    return state.placement === "left" || state.placement === "right";
  }

  private createNotch(): HTMLElement {
    const notch = ElementFactory.create("div", {
      className: "island-notch",
      children: [
        ElementFactory.create("span", { className: "island-ear island-ear-start" }),
        ElementFactory.create("span", { className: "island-ear island-ear-end" }),
        this.clip,
      ],
    });
    notch.addEventListener("pointerenter", () => this.handlers?.onPointerEnter());
    notch.addEventListener("pointerleave", () => this.handlers?.onPointerLeave());
    notch.addEventListener("click", () => {
      if (notch.dataset.mode !== "expanded") {
        this.handlers?.onExpand();
      }
    });
    return notch;
  }

  /** The content changed size by itself: the notch follows, and the click area too. */
  private followContent(size: ContentSize): void {
    if (size.width === this.notchSize.width && size.height === this.notchSize.height) {
      return;
    }
    this.resizeNotch(size);
    this.handlers?.onContentResize();
  }

  /** Gives the notch its new size; CSS animates growing and shrinking with different curves. */
  private resizeNotch(size: ContentSize): void {
    const growing = size.width * size.height >= this.notchSize.width * this.notchSize.height;
    this.notch.dataset.motion = growing ? "opening" : "closing";
    this.notchSize = size;
    this.notch.style.width = `${size.width}px`;
    this.notch.style.height = `${size.height}px`;
  }

  private content(
    state: IslandState,
    mood: MoodName,
    preferences: UserPreferences,
    handlers: IslandViewHandlers,
  ): HTMLElement {
    if (state.mode === "hidden") {
      return this.notchView.hidden(state.snapshot, mood, state.unreadAlert);
    }
    if (state.mode === "alert" && state.alert) {
      return this.notchView.alert(state.snapshot, state.alert, mood);
    }
    if (state.mode === "compact") {
      return this.notchView.compact(state.snapshot, mood);
    }
    return ElementFactory.create("div", {
      className: "island-expanded",
      children: [
        this.header.render(
          ElementFactory.fromTrustedMarkup(
            "island-head-face",
            this.mascot.render(mood, state.snapshot.moodLabel, UI_CONFIG.faceWidth.header),
          ),
          state.panel,
          this.tabLabels,
          state.tab,
          handlers,
        ),
        ElementFactory.create("div", {
          className: "island-card",
          attributes: { "data-mood": mood, "data-panel": state.panel, "data-tab": state.tab },
          children: [
            ElementFactory.create("div", {
              className: "island-card-body",
              children: [this.cardBody(state, preferences, handlers)],
            }),
          ],
        }),
      ],
    });
  }

  private cardBody(state: IslandState, preferences: UserPreferences, handlers: IslandViewHandlers): HTMLElement {
    switch (state.panel) {
      case "capture":
        return this.captureView.render(handlers.capture, state.projects, this.assistantReady);
      case "settings":
        return this.settings.render(preferences, handlers);
      case "edit":
        return state.editing
          ? this.taskEdit.render(state.editing, state.projects, handlers.edit)
          : this.tabContent(state, handlers);
      case "projects":
        return this.projectsPanel.render(state.projects, state.projectColors, state.creatingProject, handlers);
      case "brief":
        return this.receipt.render(state.snapshot, state.alert, state.dayBrief);
      case "tabs":
        return this.tabContent(state, handlers);
    }
  }

  private tabContent(state: IslandState, handlers: IslandViewHandlers): HTMLElement {
    return this.tabs[state.tab].render(state, handlers);
  }
}
