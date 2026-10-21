import type { MoodName, UserPreferences } from "../backend/contracts";
import type { CaptureView, CaptureViewHandlers } from "../capture/CaptureView";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { MascotRenderer } from "../mascot/MascotRenderer";
import type { AiSettingsHandlers } from "./AiSettingsView";
import { HeaderView, type HeaderHandlers } from "./HeaderView";
import type { IslandMode, IslandState } from "./islandState";
import { NotchView } from "./NotchView";
import { SettingsView } from "./SettingsView";
import { TaskEditView, type TaskEditHandlers } from "./TaskEditView";
import { TaskListView, type TaskListHandlers } from "./TaskListView";
import type { HitArea } from "../backend/contracts";
import { ContentTransition, type ContentSize } from "./ContentTransition";

export interface IslandViewHandlers extends HeaderHandlers, AiSettingsHandlers, TaskListHandlers {
  onExpand: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
  capture: CaptureViewHandlers;
  edit: TaskEditHandlers;
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
  private readonly notch: HTMLElement;
  private readonly clip: HTMLElement;
  private readonly contentTransition: ContentTransition;
  private handlers: IslandViewHandlers | null = null;
  private notchSize: ContentSize = { width: 0, height: 0 };
  private shownMode: IslandMode | null = null;
  private shownPanel = "";

  constructor(
    private readonly root: HTMLElement,
    private readonly translator: Translator,
    private readonly mascot: MascotRenderer,
    private readonly captureView: CaptureView,
  ) {
    this.notchView = new NotchView(mascot);
    this.header = new HeaderView(translator);
    this.taskList = new TaskListView(translator);
    this.settings = new SettingsView(translator);
    this.taskEdit = new TaskEditView(translator);
    this.clip = ElementFactory.create("div", { className: "island-clip" });
    this.notch = this.createNotch();
    this.contentTransition = new ContentTransition(this.clip);
    root.style.setProperty("--island-open-duration", `${UI_CONFIG.motion.openMs}ms`);
    root.style.setProperty("--island-close-duration", `${UI_CONFIG.motion.closeMs}ms`);
    root.style.setProperty("--island-content-out-duration", `${UI_CONFIG.motion.contentOutMs}ms`);
    root.replaceChildren(this.notch);
  }

  render(state: IslandState, preferences: UserPreferences, handlers: IslandViewHandlers): void {
    this.handlers = handlers;
    const mood = state.reactionMood ?? state.snapshot.mood;
    this.root.dataset.placement = state.placement;
    this.root.dataset.orientation = IslandView.isVertical(state) ? "vertical" : "horizontal";
    this.root.ownerDocument.body.dataset.placement = state.placement;
    this.root.ownerDocument.documentElement.lang = this.translator.localeCode;
    this.notch.dataset.mode = state.mode;
    this.notch.dataset.mood = mood;

    // Content cross-fades only when the island changes shape (hidden, compact, expanded).
    // Tabs and panels swap instantly, so switching them never flashes an empty card.
    const animate = this.shownMode !== null && state.mode !== this.shownMode;
    const panelKey = `${state.mode}:${state.panel}`;
    if (panelKey !== this.shownPanel) {
      this.captureView.reset();
      this.taskEdit.reset();
    }
    this.shownMode = state.mode;
    this.shownPanel = panelKey;

    const content = this.content(state, mood, preferences, handlers);
    this.resizeNotch(this.contentTransition.swap(content, animate));
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

  stop(): void {
    this.contentTransition.stop();
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
      return this.notchView.hidden(state.snapshot, mood);
    }
    if (state.mode === "compact") {
      return this.notchView.compact(state.snapshot, mood);
    }
    return ElementFactory.create("div", {
      className: "island-expanded",
      children: [
        this.header.render(state.panel, state.tab, handlers),
        ElementFactory.create("div", {
          className: "island-card",
          attributes: { "data-mood": mood },
          children: [
            ElementFactory.fromTrustedMarkup(
              "island-card-mascot",
              this.mascot.render(mood, state.snapshot.moodLabel, UI_CONFIG.mascotSize.card),
            ),
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
        return this.captureView.render(handlers.capture);
      case "settings":
        return this.settings.render(preferences, state.aiSettings, handlers);
      case "edit":
        return state.editing
          ? this.taskEdit.render(state.editing.task, state.editing.clientNames, handlers.edit)
          : this.tabContent(state, handlers);
      case "tasks":
        return this.tabContent(state, handlers);
    }
  }

  private tabContent(state: IslandState, handlers: IslandViewHandlers): HTMLElement {
    const subtitle = ElementFactory.create("p", { className: "island-subtitle", text: state.snapshot.todaySubtitle });
    const list = state.tab === "today"
      ? this.taskList.render(state.snapshot.sections, this.translator.text("island.today.empty"), handlers)
      : this.taskList.render([], this.translator.text(`island.${state.tab}.empty`), handlers);
    return ElementFactory.create("div", { className: "island-tab-content", children: [subtitle, list] });
  }
}
