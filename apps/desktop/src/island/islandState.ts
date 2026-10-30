import type {
  ProjectSummary,
  DeliveredItem,
  IslandPlacement,
  IslandSnapshot,
  MoodName,
  TaskDetails,
  TikiAlert,
  SystemStats,
  TrashedItem,
} from "../backend/contracts";

/**
 * How much of the island is visible, like the notch of dooai. `alert` is the notch grown a
 * little to show an alert.
 */
export const ISLAND_MODES = Object.freeze(["hidden", "compact", "alert", "expanded"] as const);
export type IslandMode = (typeof ISLAND_MODES)[number];

/**
 * What the expanded island shows in its card: the tabs, or a panel such as quick capture,
 * a task being edited, the project list or the brief drawn as a receipt.
 */
export type IslandPanel = "tabs" | "capture" | "settings" | "edit" | "projects" | "brief";

/** Pending tasks, the delivered history or the trash, in the Tasks tab. Reminders live in the Agenda. */
export const TASKS_VIEWS = Object.freeze(["pending", "delivered", "trash"] as const);
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
  /** Items in the trash, most recently trashed first. */
  trashed: TrashedItem[];
  /** Days an item stays in the trash before it is deleted forever. */
  trashRetentionDays: number;
  /** The latest measure of the computer, for the resting notch; `null` until the first one. */
  systemStats: SystemStats | null;
  /** Pending or delivered, in the Tasks tab. */
  tasksView: TasksView;
  /** A short-lived mood (happy after capturing) that overrides the snapshot's mood. */
  reactionMood: MoodName | null;
  /** The latest alert: shown in the notch, then above the Today list until seen. */
  alert: TikiAlert | null;
  /** An alert arrived and the island has not been opened since: the notch shows a dot. */
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
  trashed: [],
  trashRetentionDays: 0,
  systemStats: null,
  tasksView: "pending",
  reactionMood: null,
  alert: null,
  unreadAlert: false,
  dayBrief: null,
  preparingBrief: false,
  editing: null,
});
