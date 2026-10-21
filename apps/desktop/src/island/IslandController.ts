import type { IslandSnapshot, MoodName, TikiBackend, UserPreferences } from "../backend/contracts";
import type { CaptureView } from "../capture/CaptureView";
import { SHELL_EVENTS, UI_CONFIG } from "../config/uiConfig";
import type { IslandMode, IslandPanel, IslandState, IslandTab } from "./islandState";
import type { IslandView, IslandViewHandlers } from "./IslandView";

/** Called when the user picks another language; the app reloads its translations. */
export type LocaleChangedListener = () => void;

/** Owns the island state, reacts to the user and the shell, and asks the view to redraw. */
export class IslandController {
  private state: IslandState;
  private preferences: UserPreferences;
  private readonly timers = new Map<string, ReturnType<typeof setTimeout>>();
  private refreshTimer: ReturnType<typeof setInterval> | null = null;
  private readonly stopListening: Array<() => void> = [];

  constructor(
    private readonly backend: TikiBackend,
    private readonly view: IslandView,
    private readonly captureView: CaptureView,
    initialSnapshot: IslandSnapshot,
    initialPreferences: UserPreferences,
    private readonly onLocaleChanged: LocaleChangedListener,
  ) {
    this.preferences = initialPreferences;
    this.state = {
      mode: "hidden",
      panel: "tasks",
      tab: "today",
      placement: initialPreferences.islandPlacement,
      snapshot: initialSnapshot,
      reactionMood: null,
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
  }

  /** Stops timers and shell listeners. */
  stop(): void {
    this.timers.forEach((timer) => clearTimeout(timer));
    this.timers.clear();
    if (this.refreshTimer) {
      clearInterval(this.refreshTimer);
    }
    this.stopListening.splice(0).forEach((unlisten) => unlisten());
  }

  async refresh(): Promise<void> {
    this.update({ snapshot: await this.backend.islandSnapshot() });
  }

  toggle(): void {
    this.setMode(this.state.mode === "expanded" ? "hidden" : "expanded");
  }

  setMode(mode: IslandMode): void {
    this.cancel("fold");
    this.update({ mode, panel: mode === "expanded" ? this.state.panel : "tasks" });
  }

  openPanel(panel: IslandPanel): void {
    this.update({ mode: "expanded", panel });
  }

  selectTab(tab: IslandTab): void {
    this.update({ tab, panel: "tasks", mode: "expanded" });
  }

  async submitCapture(sentence: string): Promise<void> {
    await this.backend.capture(sentence);
    await this.refresh();
    this.openPanel("tasks");
    this.react("happy");
  }

  async deliverTask(taskId: number): Promise<void> {
    await this.backend.moveTask(taskId, "done");
    await this.refresh();
    this.react("happy");
  }

  async changePreferences(preferences: UserPreferences): Promise<void> {
    const localeChanged = preferences.locale !== this.preferences.locale;
    await this.backend.updatePreferences(preferences);
    this.preferences = preferences;
    if (localeChanged) {
      this.onLocaleChanged();
      return;
    }
    this.update({ placement: preferences.islandPlacement });
  }

  /** Hovering the hidden notch shows the compact island for a moment, like dooai. */
  private peek(): void {
    this.cancel("fold");
    if (this.state.mode === "hidden") {
      this.schedule("peek", UI_CONFIG.peekDelayMs, () => this.update({ mode: "compact" }));
    }
  }

  private fold(): void {
    this.cancel("peek");
    if (this.state.mode === "compact") {
      this.schedule("fold", UI_CONFIG.foldDelayMs, () => this.update({ mode: "hidden" }));
    }
  }

  private previewCapture(sentence: string): void {
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
      onChangePreferences: (preferences) => void this.changePreferences(preferences),
      capture: {
        onInput: (sentence) => this.previewCapture(sentence),
        onSubmit: (sentence) => void this.submitCapture(sentence),
        onCancel: () => this.openPanel("tasks"),
      },
    };
  }
}
