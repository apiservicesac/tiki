import type { MoodName, UserPreferences } from "../backend/contracts";
import type { CaptureView, CaptureViewHandlers } from "../capture/CaptureView";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { MascotRenderer } from "../mascot/MascotRenderer";
import { HeaderView, type HeaderHandlers } from "./HeaderView";
import type { IslandState } from "./islandState";
import { NotchView } from "./NotchView";
import { SettingsView } from "./SettingsView";
import { TaskListView } from "./TaskListView";
import type { IslandSize } from "./WindowFitter";

export interface IslandViewHandlers extends HeaderHandlers {
  onExpand: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
  onDeliverTask: (taskId: number) => void;
  onChangePreferences: (preferences: UserPreferences) => void;
  capture: CaptureViewHandlers;
}

/**
 * Draws the island for a given state. The notch element lives for the whole session and
 * only its content changes, so its size and corners animate between states like dooai.
 */
export class IslandView {
  private readonly notchView: NotchView;
  private readonly header: HeaderView;
  private readonly taskList: TaskListView;
  private readonly settings: SettingsView;
  private readonly notch: HTMLElement;
  private readonly clip: HTMLElement;
  private handlers: IslandViewHandlers | null = null;
  private notchSize: IslandSize = { width: 0, height: 0 };

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
    this.clip = ElementFactory.create("div", { className: "island-clip" });
    this.notch = this.createNotch();
    root.style.setProperty("--island-transition", `${UI_CONFIG.islandTransitionMs}ms`);
    root.replaceChildren(this.notch);
  }

  render(state: IslandState, preferences: UserPreferences, handlers: IslandViewHandlers): void {
    this.handlers = handlers;
    const mood = state.reactionMood ?? state.snapshot.mood;
    this.root.dataset.placement = state.placement;
    this.root.ownerDocument.body.dataset.placement = state.placement;
    this.notch.dataset.mode = state.mode;
    this.notch.dataset.mood = mood;

    const content = this.content(state, mood, preferences, handlers);
    content.classList.add("island-content");
    this.clip.replaceChildren(content);
    this.resizeNotchTo(content);
  }

  /** Size the window needs: the notch at its target size plus the room around it. */
  windowSize(): IslandSize {
    const padding = getComputedStyle(this.root);
    const horizontal = parseFloat(padding.paddingLeft) + parseFloat(padding.paddingRight);
    const vertical = parseFloat(padding.paddingTop) + parseFloat(padding.paddingBottom);
    return {
      width: Math.ceil(this.notchSize.width + (horizontal || 0)),
      height: Math.ceil(this.notchSize.height + (vertical || 0)),
    };
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

  /** Gives the notch the natural size of its new content; CSS animates the change. */
  private resizeNotchTo(content: HTMLElement): void {
    this.notchSize = { width: content.offsetWidth, height: content.offsetHeight };
    this.notch.style.width = `${this.notchSize.width}px`;
    this.notch.style.height = `${this.notchSize.height}px`;
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
        return this.settings.render(preferences, handlers.onChangePreferences);
      case "tasks":
        return this.tabContent(state, handlers);
    }
  }

  private tabContent(state: IslandState, handlers: IslandViewHandlers): HTMLElement {
    const subtitle = ElementFactory.create("p", { className: "island-subtitle", text: state.snapshot.todaySubtitle });
    const list = state.tab === "today"
      ? this.taskList.render(state.snapshot.sections, this.translator.text("island.today.empty"), handlers.onDeliverTask)
      : this.taskList.render([], this.translator.text(`island.${state.tab}.empty`), handlers.onDeliverTask);
    return ElementFactory.create("div", { className: "island-tab-content", children: [subtitle, list] });
  }
}
