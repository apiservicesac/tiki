import type { InterpretedCapture } from "../ai";
import { AssistantActions } from "../assistant/AssistantActions";
import type { AskView } from "../assistant/AskView";
import type { TikiAssistant } from "../assistant/TikiAssistant";
import {
  LOCALE_CODES,
  type IslandSnapshot,
  type MoodName,
  type TaskEdit,
  type TikiBackend,
  type UserPreferences,
} from "../backend/contracts";
import type { CaptureView } from "../capture/CaptureView";
import type { Translator } from "../i18n/Translator";
import { SHELL_EVENTS, UI_CONFIG } from "../config/uiConfig";
import type { IslandMode, IslandPanel, IslandState, IslandTab } from "./islandState";
import type { IslandView, IslandViewHandlers } from "./IslandView";
import type { HitAreaReporter } from "./HitAreaReporter";
import type { IslandStore } from "./IslandStore";

/** The views the controller drives directly, besides the island itself. */
export interface IslandViews {
  island: IslandView;
  capture: CaptureView;
  ask: AskView;
}

/** Owns the island state, reacts to the user and the shell, and asks the view to redraw. */
export class IslandController {
  private state: IslandState;
  private preferences: UserPreferences;
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private refreshTimer: ReturnType<typeof setInterval> | null = null;
  /** What the assistant understood from the sentence in the box, waiting for a second Enter. */
  private pendingCapture: { sentence: string; capture: InterpretedCapture } | null = null;
  private readonly assistantActions: AssistantActions;
  private readonly view: IslandView;
  private readonly captureView: CaptureView;
  private readonly askView: AskView;
  private readonly stopListening: Array<() => void> = [];
  /** The window changes size when the island moves to another orientation. */
  private readonly redrawOnResize = () => this.render();

  constructor(
    private readonly backend: TikiBackend,
    views: IslandViews,
    private readonly hitAreaReporter: HitAreaReporter,
    initialSnapshot: IslandSnapshot,
    initialPreferences: UserPreferences,
    private readonly translator: Translator,
    private readonly assistant: TikiAssistant,
  ) {
    this.view = views.island;
    this.captureView = views.capture;
    this.askView = views.ask;
    this.preferences = initialPreferences;
    this.state = {
      mode: "hidden",
      panel: "tasks",
      tab: "today",
      placement: initialPreferences.islandPlacement,
      snapshot: initialSnapshot,
      reactionMood: null,
      editing: null,
      dayBrief: null,
    };
    this.assistantActions = new AssistantActions(this.store(), assistant, views.ask);
  }

  get currentState(): Readonly<IslandState> {
    return this.state;
  }

  /** Draws the first frame and starts listening to the shell. */
  async start(): Promise<void> {
    this.render();
    this.stopListening.push(
      await this.backend.listen(SHELL_EVENTS.openCapture, () => this.openPanel("capture")),
      await this.backend.listen(SHELL_EVENTS.openSettings, () => this.openPanel("settings")),
      await this.backend.listen(SHELL_EVENTS.toggleIsland, () => this.toggle()),
      await this.backend.listen(SHELL_EVENTS.briefReady, () => this.announceBrief()),
      await this.backend.listen(SHELL_EVENTS.preferencesChanged, () => void this.reloadPreferences()),
    );
    this.refreshTimer = setInterval(() => void this.refresh(), UI_CONFIG.snapshotRefreshIntervalMs);
    window.addEventListener("resize", this.redrawOnResize);
    await this.loadAssistantStatus();
  }

  /** Stops timers and shell listeners. */
  stop(): void {
    this.timers.forEach((timer) => clearTimeout(timer));
    this.timers.clear();
    this.hitAreaReporter.stop();
    this.view.stop();
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
    }
    this.stopListening.splice(0).forEach((unlisten) => unlisten());
    window.removeEventListener("resize", this.redrawOnResize);
  }

  async refresh(): Promise<void> {
    this.update({ snapshot: await this.backend.islandSnapshot() });
  }

  toggle(): void {
    this.setMode(this.state.mode === "expanded" ? "hidden" : "expanded");
  }

  setMode(mode: IslandMode): void {
    this.cancel("fold");
    this.cancel("unfold");
    this.update({ mode, panel: mode === "expanded" ? this.state.panel : "tasks" });
  }

  openPanel(panel: IslandPanel): void {
    this.update({ mode: "expanded", panel });
  }

  selectTab(tab: IslandTab): void {
    this.update({ tab, panel: "tasks", mode: "expanded" });
  }

  /**
   * With an AI key, the first Enter shows what the assistant understood and the second one
   * saves it. Without a key, for `+Name`, or when the assistant fails, the core's own rules
   * save the sentence directly.
   */
  async submitCapture(sentence: string): Promise<void> {
    const pending = this.pendingCapture;
    if (pending?.sentence === sentence) {
      this.pendingCapture = null;
      await this.backend.saveConfirmedCapture(pending.capture);
      await this.finishCapture();
      return;
    }
    if (sentence.trim().startsWith("+") || !this.assistant.isReady(this.preferences)) {
      await this.captureWithRules(sentence);
      return;
    }
    this.cancel("preview");
    this.captureView.showThinking();
    try {
      const capture = await this.assistant.interpret(sentence, this.preferences);
      this.pendingCapture = { sentence, capture };
      this.captureView.showInterpreted(capture);
    } catch {
      await this.captureWithRules(sentence);
    }
  }

  /** Opens a task in the edit panel. */
  async openTask(taskId: number): Promise<void> {
    const [task, clientNames] = await Promise.all([this.backend.taskDetails(taskId), this.backend.clientNames()]);
    this.update({ mode: "expanded", panel: "edit", editing: { task, clientNames } });
  }

  async saveTask(taskId: number, edit: TaskEdit): Promise<void> {
    await this.backend.editTask(taskId, edit);
    await this.refresh();
    this.update({ panel: "tasks", editing: null });
    this.react(edit.status === "done" ? "happy" : "curious");
  }

  async deliverTask(taskId: number): Promise<void> {
    await this.backend.moveTask(taskId, "done");
    await this.refresh();
    this.react("happy");
  }

  /** Folds the island back to its notch, so it never covers the advanced settings window. */
  openAdvancedSettings(section?: string): void {
    this.setMode("hidden");
    void this.backend.openAdvancedSettings(section);
  }

  /** Saves the preferences and applies them in place, without closing the island. */
  async changePreferences(preferences: UserPreferences): Promise<void> {
    await this.backend.updatePreferences(preferences);
    await this.applyPreferences(preferences);
  }

  /** Preferences or AI keys changed, maybe in the advanced settings window. */
  async reloadPreferences(): Promise<void> {
    await this.applyPreferences(await this.backend.preferences());
    await this.loadAssistantStatus();
  }

  private async applyPreferences(preferences: UserPreferences): Promise<void> {
    const localeChanged = preferences.locale !== this.preferences.locale;
    this.preferences = preferences;
    if (localeChanged) {
      this.translator.use(LOCALE_CODES[preferences.locale]);
      this.captureView.reset();
      this.askView.reset();
    }
    this.update({ placement: preferences.islandPlacement, dayBrief: localeChanged ? null : this.state.dayBrief });
    await this.refresh();
  }

  /** Which providers have a key, so Ask Tiki and AI capture know whether they can work. */
  private async loadAssistantStatus(): Promise<void> {
    await this.assistant.loadStatus();
    this.render();
  }

  /**
   * A brief notification was just shown: the hidden island peeks out with the day's summary
   * for a moment, then hides again unless the user is already on it.
   */
  private async announceBrief(): Promise<void> {
    await this.refresh();
    void this.assistantActions.prepareDayBrief();
    if (this.state.mode !== "hidden") {
      return;
    }
    this.update({ mode: "compact" });
    this.react("curious");
    this.schedule("fold", UI_CONFIG.briefPeekDurationMs, () => {
      if (this.state.mode === "compact") {
        this.update({ mode: "hidden" });
      }
    });
  }

  private async captureWithRules(sentence: string): Promise<void> {
    await this.backend.capture(sentence);
    await this.finishCapture();
  }

  private async finishCapture(): Promise<void> {
    this.captureView.reset();
    await this.refresh();
    this.openPanel("tasks");
    this.react("happy");
  }

  /**
   * Resting the pointer on the notch opens it in two steps, like dooai: first the compact
   * island with the day's summary, then the whole island. No click needed.
   */
  private peek(): void {
    this.cancel("fold");
    if (this.state.mode !== "hidden") {
      return;
    }
    this.schedule("peek", UI_CONFIG.peekDelayMs, () => {
      this.update({ mode: "compact" });
      this.schedule("unfold", UI_CONFIG.unfoldDelayMs, () => this.update({ mode: "expanded" }));
    });
  }

  /** Leaving the island folds it back, unless the user is typing or changing settings. */
  private fold(): void {
    this.cancel("peek");
    this.cancel("unfold");
    if (this.state.mode === "hidden" || this.state.panel !== "tasks") {
      return;
    }
    this.schedule("fold", UI_CONFIG.foldDelayMs, () => this.update({ mode: "hidden", panel: "tasks" }));
  }

  private previewCapture(sentence: string): void {
    this.pendingCapture = null;
    if (sentence.trim() === "") {
      this.cancel("preview");
      this.captureView.showIntent(null);
      return;
    }
    this.schedule("preview", UI_CONFIG.capturePreviewDelayMs, async () => {
      this.captureView.showIntent(await this.backend.previewCapture(sentence));
    });
  }

  /** Shows a reaction mood for a moment, then goes back to the resting mood. */
  private react(mood: MoodName): void {
    this.update({ reactionMood: mood });
    this.schedule("reaction", UI_CONFIG.reactionMoodDurationMs, () => this.update({ reactionMood: null }));
  }

  private schedule(name: string, delayMs: number, action: () => void): void {
    this.cancel(name);
    this.timers.set(name, setTimeout(() => {
      this.timers.delete(name);
      action();
    }, delayMs));
  }

  private cancel(name: string): void {
    const timer = this.timers.get(name);
    if (timer) {
      clearTimeout(timer);
      this.timers.delete(name);
    }
  }

  /** What the action helpers may read and change. */
  private store(): IslandStore {
    return {
      state: () => this.state,
      preferences: () => this.preferences,
      update: (changes) => this.update(changes),
    };
  }

  private update(changes: Partial<IslandState>): void {
    this.state = { ...this.state, ...changes };
    this.render();
  }

  private render(): void {
    this.view.render(this.state, this.preferences, this.handlers(), this.assistantActions.ready);
    this.hitAreaReporter.report(this.view.hitArea());
  }

  private handlers(): IslandViewHandlers {
    return {
      onExpand: () => this.setMode("expanded"),
      onPointerEnter: () => this.peek(),
      onPointerLeave: () => this.fold(),
      onSelectTab: (tab) => this.selectTab(tab),
      onOpenPanel: (panel) => this.openPanel(panel),
      onClose: () => this.setMode("hidden"),
      onDeliverTask: (taskId) => void this.deliverTask(taskId),
      onOpenTask: (taskId) => void this.openTask(taskId),
      onChangePreferences: (preferences) => void this.changePreferences(preferences),
      onOpenAdvancedSettings: () => this.openAdvancedSettings(),
      ask: {
        onAsk: (question) => void this.assistantActions.ask(question),
        onSummarize: () => void this.assistantActions.summarize(),
        onOpenSettings: () => this.openAdvancedSettings("ai"),
        onCancel: () => this.openPanel("tasks"),
      },
      edit: {
        onSave: (taskId, edit) => void this.saveTask(taskId, edit),
        onCancel: () => this.update({ panel: "tasks", editing: null }),
      },
      capture: {
        onInput: (sentence) => this.previewCapture(sentence),
        onSubmit: (sentence) => void this.submitCapture(sentence),
        onCancel: () => {
          this.pendingCapture = null;
          this.openPanel("tasks");
        },
      },
    };
  }
}
