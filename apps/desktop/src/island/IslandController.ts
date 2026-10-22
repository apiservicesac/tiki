import { AssistantActions } from "../assistant/AssistantActions";
import type { AskView } from "../assistant/AskView";
import type { TikiAssistant } from "../assistant/TikiAssistant";
import {
  LOCALE_CODES,
  type TikiAlert,
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
import { AlertSound } from "./AlertSound";
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
  private readonly assistantActions: AssistantActions;
  private readonly sound = new AlertSound();
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
      projects: [],
      projectColors: [],
      delivered: [],
      tasksView: "pending",
      creatingProject: false,
      reactionMood: null,
      editing: null,
      dayBrief: null,
      alert: null,
      unreadAlert: false,
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
      await this.backend.onAlert((brief) => void this.announceAlert(brief)),
      await this.backend.listen(SHELL_EVENTS.preferencesChanged, () => void this.reloadPreferences()),
    );
    this.refreshTimer = setInterval(() => void this.refresh(), UI_CONFIG.snapshotRefreshIntervalMs);
    window.addEventListener("resize", this.redrawOnResize);
    await this.loadAssistantStatus();
    this.update({ projectColors: await this.backend.projectColors() });
    await this.refresh();
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

  /** The day, the projects and the history, fresh from the core. */
  async refresh(): Promise<void> {
    const [snapshot, projects, delivered] = await Promise.all([
      this.backend.islandSnapshot(),
      this.backend.projectSummaries(),
      this.backend.deliveredTasks(),
    ]);
    this.update({ snapshot, projects, delivered });
  }

  toggle(): void {
    this.setMode(this.state.mode === "expanded" ? "hidden" : "expanded");
  }

  setMode(mode: IslandMode): void {
    this.cancel("alert");
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
   * Enter in quick capture. With an AI key, the first Enter fills the fields from the
   * sentence for the user to review, and the next one saves. Without a key, the sentence is
   * the title and the user picks the fields, so Enter saves right away.
   */
  async submitCapture(): Promise<void> {
    if (!this.captureView.isInterpreted && this.assistant.isReady(this.preferences)) {
      const sentence = this.captureView.values().title;
      this.captureView.showThinking();
      try {
        this.captureView.fill(await this.assistant.interpret(sentence, this.preferences));
      } catch {
        this.captureView.showAiFailed();
      }
      return;
    }
    const capture = this.captureView.values();
    if (capture.title === "") {
      return;
    }
    await this.backend.saveConfirmedCapture(capture);
    this.captureView.reset();
    await this.refresh();
    this.openPanel("tasks");
    this.react("happy");
  }

  /** Opens a task in the edit panel. */
  async openTask(taskId: number): Promise<void> {
    const task = await this.backend.taskDetails(taskId);
    this.update({ mode: "expanded", panel: "edit", editing: task });
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

  /**
   * Opens what an alert is about: the task of a task reminder, or the brief drawn as a
   * receipt for the morning brief and the end-of-day close.
   */
  async openAlert(): Promise<void> {
    const alert = this.state.alert;
    this.cancel("alert");
    if (alert?.kind === "task" && alert.taskId !== null) {
      await this.openTask(alert.taskId);
      return;
    }
    this.update({ mode: "expanded", panel: alert ? "brief" : "tasks" });
  }

  /** Creates, changes or removes a project, then shows the result; a taken name is ignored. */
  private async projectAction(action: () => Promise<unknown>): Promise<void> {
    try {
      await action();
    } catch {
      this.react("worried");
    }
    await this.refresh();
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
   * A reminder arrived: the notch grows a little with Tiki, the title and a preview, and
   * chimes. It folds back after a moment, leaving a dot until the island is opened. If the
   * user already has the island open, the reminder just appears above the Today list.
   */
  async announceAlert(brief: TikiAlert): Promise<void> {
    await this.refresh();
    void this.assistantActions.prepareDayBrief();
    const alreadyOpen = this.state.mode === "expanded";
    this.update({ alert: brief, unreadAlert: !alreadyOpen });
    if (this.preferences.alertSound) {
      this.sound.play();
    }
    this.react("curious");
    if (alreadyOpen) {
      return;
    }
    ["peek", "unfold", "fold"].forEach((timer) => this.cancel(timer));
    this.update({ mode: "alert" });
    this.schedule("alert", UI_CONFIG.alertDurationMs, () => {
      if (this.state.mode === "alert") {
        this.update({ mode: "hidden" });
      }
    });
  }

  /**
   * Resting the pointer on the notch opens it in two steps, like dooai: first the compact
   * island with the day's summary, then the whole island. No click needed.
   */
  private peek(): void {
    this.cancel("fold");
    if (this.state.mode === "alert") {
      // The reminder is already showing: resting on it opens the whole island.
      this.cancel("alert");
      this.schedule("unfold", UI_CONFIG.peekDelayMs, () => this.update({ mode: "expanded" }));
      return;
    }
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
    if (this.state.mode === "hidden" || this.state.panel !== "tasks" || this.view.isTyping()) {
      return;
    }
    this.schedule("fold", UI_CONFIG.foldDelayMs, () => this.update({ mode: "hidden", panel: "tasks" }));
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
    const next = { ...this.state, ...changes };
    // Opening the island reads the reminder; folding it afterwards puts the reminder away.
    if (next.mode === "expanded") {
      next.unreadAlert = false;
    } else if (next.mode === "hidden" && this.state.mode === "expanded") {
      next.alert = null;
    }
    this.state = next;
    this.render();
  }

  private render(): void {
    this.view.render(this.state, this.preferences, this.handlers(), this.assistantActions.ready);
    this.hitAreaReporter.report(this.view.hitArea());
  }

  private handlers(): IslandViewHandlers {
    return {
      onExpand: () => (this.state.mode === "alert" ? void this.openAlert() : this.setMode("expanded")),
      onOpenAlert: () => void this.openAlert(),
      onShowDelivered: () => this.update({ panel: "tasks", tab: "all_tasks", tasksView: "delivered" }),
      onSelectTasksView: (tasksView) => this.update({ tasksView }),
      onManageProjects: (creatingProject) => this.update({ mode: "expanded", panel: "projects", creatingProject }),
      onCreateProject: (name) => void this.projectAction(() => this.backend.createProject(name)),
      onChangeProject: (projectId, change) => void this.projectAction(() => this.backend.changeProject(projectId, change)),
      onRemoveProject: (projectId) => void this.projectAction(() => this.backend.removeProject(projectId)),
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
        onSubmit: () => void this.submitCapture(),
        onCancel: () => this.openPanel("tasks"),
        onOpenAiSettings: () => this.openAdvancedSettings("ai"),
      },
    };
  }
}
