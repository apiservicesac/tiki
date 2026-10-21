import type { AiProviderName, InterpretedCapture } from "../ai";
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

/** Owns the island state, reacts to the user and the shell, and asks the view to redraw. */
export class IslandController {
  private state: IslandState;
  private preferences: UserPreferences;
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private refreshTimer: ReturnType<typeof setInterval> | null = null;
  /** What the assistant understood from the sentence in the box, waiting for a second Enter. */
  private pendingCapture: { sentence: string; capture: InterpretedCapture } | null = null;
  /** Counts AI settings loads, so a slow older load never overwrites a newer one. */
  private aiSettingsLoads = 0;
  private readonly stopListening: Array<() => void> = [];
  /** The window changes size when the island moves to another orientation. */
  private readonly redrawOnResize = () => this.render();

  constructor(
    private readonly backend: TikiBackend,
    private readonly view: IslandView,
    private readonly captureView: CaptureView,
    private readonly hitAreaReporter: HitAreaReporter,
    initialSnapshot: IslandSnapshot,
    initialPreferences: UserPreferences,
    private readonly translator: Translator,
    private readonly assistant: TikiAssistant,
  ) {
    this.preferences = initialPreferences;
    this.state = {
      mode: "hidden",
      panel: "tasks",
      tab: "today",
      placement: initialPreferences.islandPlacement,
      snapshot: initialSnapshot,
      reactionMood: null,
      aiSettings: { configured: { anthropic: false, openai: false }, models: [] },
      editing: null,
    };
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
    );
    this.refreshTimer = setInterval(() => void this.refresh(), UI_CONFIG.snapshotRefreshIntervalMs);
    window.addEventListener("resize", this.redrawOnResize);
    await this.loadAiSettings();
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
    if (panel === "settings") {
      void this.loadAiSettings();
    }
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

  async saveAiKey(provider: AiProviderName, apiKey: string): Promise<void> {
    await this.backend.saveAiKey(provider, apiKey);
    await this.loadAiSettings();
  }

  async removeAiKey(provider: AiProviderName): Promise<void> {
    await this.backend.removeAiKey(provider);
    await this.loadAiSettings();
  }

  /** Saves the preferences and applies them in place, without closing the island. */
  async changePreferences(preferences: UserPreferences): Promise<void> {
    const localeChanged = preferences.locale !== this.preferences.locale;
    const providerChanged = preferences.aiProvider !== this.preferences.aiProvider;
    await this.backend.updatePreferences(preferences);
    this.preferences = preferences;
    if (localeChanged) {
      this.translator.use(LOCALE_CODES[preferences.locale]);
      this.captureView.reset();
    }
    this.update({ placement: preferences.islandPlacement });
    await this.refresh();
    if (providerChanged) {
      await this.loadAiSettings();
    }
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

  /** Which providers have a key and which models the current one offers. */
  private async loadAiSettings(): Promise<void> {
    const load = ++this.aiSettingsLoads;
    const configured = await this.assistant.loadStatus();
    const models = await this.assistant.models(this.preferences.aiProvider);
    if (load === this.aiSettingsLoads) {
      this.update({ aiSettings: { configured, models } });
    }
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

  private update(changes: Partial<IslandState>): void {
    this.state = { ...this.state, ...changes };
    this.render();
  }

  private render(): void {
    this.view.render(this.state, this.preferences, this.handlers());
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
      onSaveKey: (provider, apiKey) => void this.saveAiKey(provider, apiKey),
      onRemoveKey: (provider) => void this.removeAiKey(provider),
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
