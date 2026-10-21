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

export interface IslandViewHandlers extends HeaderHandlers {
  onExpand: () => void;
  onPointerEnter: () => void;
  onPointerLeave: () => void;
  onDeliverTask: (taskId: number) => void;
  onChangePreferences: (preferences: UserPreferences) => void;
  capture: CaptureViewHandlers;
}

/** Draws the whole island for a given state; it holds no state of its own. */
export class IslandView {
  private readonly notch: NotchView;
  private readonly header: HeaderView;
  private readonly taskList: TaskListView;
  private readonly settings: SettingsView;

  constructor(
    private readonly root: HTMLElement,
    private readonly translator: Translator,
    private readonly mascot: MascotRenderer,
    private readonly captureView: CaptureView,
  ) {
    this.notch = new NotchView(mascot);
    this.header = new HeaderView(translator);
    this.taskList = new TaskListView(translator);
    this.settings = new SettingsView(translator);
  }

  render(state: IslandState, preferences: UserPreferences, handlers: IslandViewHandlers): void {
    const mood = state.reactionMood ?? state.snapshot.mood;
    const notch = ElementFactory.create("div", {
      className: "island-notch",
      attributes: { "data-mode": state.mode, "data-mood": mood },
      children: [
        ElementFactory.create("span", { className: "island-ear island-ear-start" }),
        ElementFactory.create("span", { className: "island-ear island-ear-end" }),
        this.content(state, mood, preferences, handlers),
      ],
    });

    if (state.mode !== "expanded") {
      notch.addEventListener("click", handlers.onExpand);
      notch.addEventListener("pointerenter", handlers.onPointerEnter);
      notch.addEventListener("pointerleave", handlers.onPointerLeave);
    }

    this.root.dataset.placement = state.placement;
    this.root.replaceChildren(notch);
  }

  private content(
    state: IslandState,
    mood: MoodName,
    preferences: UserPreferences,
    handlers: IslandViewHandlers,
  ): HTMLElement {
    if (state.mode === "hidden") {
      return this.notch.hidden(state.snapshot, mood);
    }
    if (state.mode === "compact") {
      return this.notch.compact(state.snapshot, mood);
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
