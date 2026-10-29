import type {
  ProjectSummary,
  DeliveredItem,
  IslandPlacement,
  IslandSnapshot,
  MoodName,
  TaskDetails,
  TikiAlert,
} from "../backend/contracts";

/**
 * How much of the island is visible, like the notch of dooai. `alert` is the notch grown a
 * little to show a reminder.
 */
export const ISLAND_MODES = Object.freeze(["hidden", "compact", "alert", "expanded"] as const);
export type IslandMode = (typeof ISLAND_MODES)[number];

/**
 * What the expanded island shows in its card: the tabs, or a panel such as quick capture,
 * a task being edited, the project list or the brief drawn as a receipt.
 */
export type IslandPanel = "tabs" | "capture" | "settings" | "edit" | "projects" | "brief";

/** Pending tasks, or the delivered history, in the Tasks tab. Reminders live in the Agenda. */
export const TASKS_VIEWS = Object.freeze(["pending", "delivered"] as const);
export type TasksView = (typeof TASKS_VIEWS)[number];

/** Tickets join as a tab when a ticket connector arrives. */
export type IslandTab = "today" | "all_tasks" | "agenda";

/** Everything the island view needs to draw one frame. */
export interface IslandState {
  mode: IslandMode;
  panel: IslandPanel;
  tab: IslandTab;
  placement: IslandPlacement;
  snapshot: IslandSnapshot;
  /** The user's projects with their workload, busiest first. */
  projects: ProjectSummary[];
  /** Colors a project can take. */
  projectColors: string[];
  /** Delivered tasks, most recent first. */
  delivered: DeliveredItem[];
  /** Pending or delivered, in the Tasks tab. */
  tasksView: TasksView;
  /** The project panel opened to create a project: its name box gets the focus. */
  creatingProject: boolean;
  /** A short-lived mood (happy after capturing) that overrides the snapshot's mood. */
  reactionMood: MoodName | null;
  /** The latest reminder: shown in the notch, then above the Today list until seen. */
  alert: TikiAlert | null;
  /** A reminder arrived and the island has not been opened since: the notch shows a dot. */
  unreadAlert: boolean;
  /** The AI brief of the day, shown in the summary card of Today. */
  dayBrief: string | null;
  /** The AI is writing the brief. */
  preparingBrief: boolean;
  /** The task open in the edit panel. */
  editing: TaskDetails | null;
}

/** How the island starts, before it knows where it sits and what the day holds. */
export const ISLAND_STATE_DEFAULTS: Readonly<Omit<IslandState, "placement" | "snapshot">> = Object.freeze({
  mode: "hidden",
  panel: "tabs",
  tab: "today",
  projects: [],
  projectColors: [],
  delivered: [],
  tasksView: "pending",
  creatingProject: false,
  reactionMood: null,
  alert: null,
  unreadAlert: false,
  dayBrief: null,
  preparingBrief: false,
  editing: null,
});
