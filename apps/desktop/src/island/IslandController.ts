import { ActionApplier } from "../assistant/ActionApplier";
import { AssistantActions } from "../assistant/AssistantActions";
import type { TikiAssistant } from "../assistant/TikiAssistant";
import type { IslandSnapshot, MoodName, TikiBackend, UserPreferences } from "../backend/contracts";
import { CaptureFlow } from "../capture/CaptureFlow";
import type { CaptureView } from "../capture/CaptureView";
import { SHELL_EVENTS, UI_CONFIG } from "../config/uiConfig";
import type { Translator } from "../i18n/Translator";
import { AlertFlow } from "./AlertFlow";
import { DelayedAction } from "./DelayedAction";
import type { HitAreaReporter } from "./HitAreaReporter";
import { IslandModeMachine } from "./IslandModeMachine";
import { ISLAND_STATE_DEFAULTS, type IslandMode, type IslandPanel, type IslandState, type IslandTab } from "./islandState";
import type { IslandStore } from "./IslandStore";
import type { IslandView, IslandViewHandlers } from "./IslandView";
import { ItemActions } from "./ItemActions";
import { PreferencesFlow } from "./PreferencesFlow";

/** The views the controller drives directly, besides the island itself. */
export interface IslandViews {
  island: IslandView;
  capture: CaptureView;
}

/**
 * Owns the island state and wires the shell, the user and the views together: the mode
 * machine decides when the island grows and folds, the capture flow and the item actions
 * do what the user asks, and every change redraws the island.
 */
export class IslandController {
  private state: IslandState;
  private refreshTimer: ReturnType<typeof setInterval> | null = null;
  private readonly reaction = new DelayedAction();
  private readonly view: IslandView;
  private readonly modes: IslandModeMachine;
  private readonly preferences: PreferencesFlow;
  private readonly assistantActions: AssistantActions;
  private readonly captureFlow: CaptureFlow;
  private readonly items: ItemActions;
  private readonly alerts: AlertFlow;
  private readonly stopListening: Array<() => void> = [];
  /** The window changes size when the island moves to another orientation. */
  private readonly redrawOnResize = () => this.render();

  constructor(
    private readonly backend: TikiBackend,
    views: IslandViews,
    private readonly hitAreaReporter: HitAreaReporter,
    initialSnapshot: IslandSnapshot,
    initialPreferences: UserPreferences,
    translator: Translator,
    assistant: TikiAssistant,
  ) {
    this.view = views.island;
    this.state = { ...ISLAND_STATE_DEFAULTS, placement: initialPreferences.islandPlacement, snapshot: initialSnapshot };
    // Leaving the island folds it, unless the user is in a panel or typing.
    this.modes = new IslandModeMachine(() => this.state.mode, (mode) => this.showMode(mode), {
      panel: () => this.state.panel !== "tabs",
      typing: () => this.view.isTyping(),
    });
    const store = this.store();
    this.preferences = new PreferencesFlow(initialPreferences, backend, translator, views.capture, assistant, store);
    this.assistantActions = new AssistantActions(store, assistant, new ActionApplier(backend));
    this.captureFlow = new CaptureFlow(views.capture, backend, assistant, this.assistantActions, store);
    this.items = new ItemActions(backend, store);
    this.alerts = new AlertFlow(store, this.modes, this.items, this.assistantActions);
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
      await this.backend.listen(SHELL_EVENTS.toggleIsland, () => this.setMode(this.state.mode === "expanded" ? "hidden" : "expanded")),
      await this.backend.onAlert((alert) => void this.alerts.announce(alert)),
      await this.backend.listen(SHELL_EVENTS.preferencesChanged, () => void this.preferences.reload()),
      await this.backend.listen(SHELL_EVENTS.calendarSynced, () => void this.refresh()),
      await this.backend.listen(SHELL_EVENTS.inCallChanged, () => void this.refresh()),
    );
    this.refreshTimer = setInterval(() => void this.refresh(), UI_CONFIG.snapshotRefreshIntervalMs);
    window.addEventListener("resize", this.redrawOnResize);
    await this.preferences.loadAssistantStatus();
    this.update({ projectColors: await this.backend.projectColors() });
    await this.refresh();
  }

  /** Stops timers and shell listeners. */
  stop(): void {
    this.modes.cancel();
    this.reaction.cancel();
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

  setMode(mode: IslandMode): void {
    this.modes.moveTo(mode);
  }

  openPanel(panel: IslandPanel): void {
    this.update({ mode: "expanded", panel });
  }

  selectTab(tab: IslandTab): void {
    this.update({ tab, panel: "tabs", mode: "expanded" });
  }

  submitCapture(): Promise<void> {
    return this.captureFlow.submit();
  }

  /** Saves the preferences and applies them in place, without closing the island. */
  changePreferences(preferences: UserPreferences): Promise<void> {
    return this.preferences.change(preferences);
  }

  /** Folds the island back to its notch, so it never covers the advanced settings window. */
  private openAdvancedSettings(section?: string): void {
    this.setMode("hidden");
    void this.backend.openAdvancedSettings(section);
  }

  /** A mode chosen by the machine or the user; leaving the expanded island goes back to the tabs. */
  private showMode(mode: IslandMode): void {
    this.update({ mode, panel: mode === "expanded" ? this.state.panel : "tabs" });
  }

  private react(mood: MoodName): void {
    this.update({ reactionMood: mood });
    this.reaction.schedule(UI_CONFIG.reactionMoodDurationMs, () => this.update({ reactionMood: null }));
  }

  /** What the flows and actions may read and change. */
  private store(): IslandStore {
    return {
      state: () => this.state,
      preferences: () => this.preferences.current,
      update: (changes) => this.update(changes),
      refresh: () => this.refresh(),
      redraw: () => this.render(),
      react: (mood) => this.react(mood),
    };
  }

  private update(changes: Partial<IslandState>): void {
    const next = { ...this.state, ...changes };
    // Opening the island reads the alert; folding it afterwards puts the alert away.
    if (next.mode === "expanded") {
      next.unreadAlert = false;
    } else if (next.mode === "hidden" && this.state.mode === "expanded") {
      next.alert = null;
    }
    this.state = next;
    this.render();
  }

  private render(): void {
    this.view.render(this.state, this.preferences.current, this.handlers(), this.assistantActions.ready);
    this.hitAreaReporter.report(this.view.hitArea());
  }

  private handlers(): IslandViewHandlers {
    return {
      onContentResize: () => this.hitAreaReporter.report(this.view.hitArea()),
      onExpand: () => (this.state.mode === "alert" ? void this.alerts.open() : this.setMode("expanded")),
      onOpenAlert: () => void this.alerts.open(),
      onDismissAlert: () => this.alerts.dismiss(),
      onSnooze: (taskId, minutes) => void this.items.snooze(taskId, minutes),
      onJoinMeeting: (url) => void this.items.joinMeeting(url),
      onPrepareBrief: () => void this.assistantActions.prepareDayBrief(),
      onShowDelivered: () => this.update({ panel: "tabs", tab: "all_tasks", tasksView: "delivered" }),
      onSelectTasksView: (tasksView) => this.update({ tasksView }),
      onManageProjects: () => this.update({ mode: "expanded", panel: "projects" }),
      onCreateProject: (name) => void this.items.createProject(name),
      onChangeProject: (projectId, change) => void this.items.changeProject(projectId, change),
      onRemoveProject: (projectId) => void this.items.removeProject(projectId),
      onPointerEnter: () => this.modes.send("pointerEnter"),
      onPointerLeave: () => this.modes.send("pointerLeave"),
      onSelectTab: (tab) => this.selectTab(tab),
      onOpenPanel: (panel) => this.openPanel(panel),
      onClose: () => this.setMode("hidden"),
      onDeliverTask: (taskId) => void this.items.deliver(taskId),
      onReopenTask: (taskId) => void this.items.reopen(taskId),
      onOpenTask: (taskId) => void this.items.open(taskId),
      onChangePreferences: (preferences) => void this.changePreferences(preferences),
      onOpenAdvancedSettings: () => this.openAdvancedSettings(),
      edit: {
        onSave: (taskId, edit) => this.items.save(taskId, edit),
        onCancel: () => this.update({ panel: "tabs", editing: null }),
      },
      capture: {
        onSubmit: () => void this.submitCapture(),
        onCancel: () => this.openPanel("tabs"),
        onOpenAiSettings: () => this.openAdvancedSettings("ai"),
      },
    };
  }
}
