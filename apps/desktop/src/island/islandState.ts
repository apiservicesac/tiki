import type {
  ProjectSummary,
  DeliveredItem,
  IslandPlacement,
  IslandSnapshot,
  MoodName,
  TaskDetails,
  TikiAlert,
} from "../backend/contracts";
import type { TasksView } from "./TasksTabView";

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
export type IslandPanel = "tasks" | "capture" | "settings" | "edit" | "projects" | "brief";

/** Tickets join as a tab when a ticket connector arrives. */
export type IslandTab = "today" | "all_tasks" | "agenda";

/** Tabs in display order, each with its translation key. */
export const ISLAND_TABS: ReadonlyArray<{ tab: IslandTab; labelKey: string }> = Object.freeze([
  { tab: "today", labelKey: "island.tab.today" },
  { tab: "all_tasks", labelKey: "island.tab.all_tasks" },
  { tab: "agenda", labelKey: "island.tab.agenda" },
]);

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
