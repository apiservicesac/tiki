import type {
  ClientSummary,
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
 * a task being edited, the client list or the brief drawn as a receipt.
 */
export type IslandPanel = "tasks" | "capture" | "settings" | "edit" | "ask" | "clients" | "brief";

export type IslandTab = "today" | "all_tasks" | "tickets" | "agenda";

/** Tabs in display order, each with its translation key. */
export const ISLAND_TABS: ReadonlyArray<{ tab: IslandTab; labelKey: string }> = Object.freeze([
  { tab: "today", labelKey: "island.tab.today" },
  { tab: "all_tasks", labelKey: "island.tab.all_tasks" },
  { tab: "tickets", labelKey: "island.tab.tickets" },
  { tab: "agenda", labelKey: "island.tab.agenda" },
]);

/** Everything the island view needs to draw one frame. */
export interface IslandState {
  mode: IslandMode;
  panel: IslandPanel;
  tab: IslandTab;
  placement: IslandPlacement;
  snapshot: IslandSnapshot;
  /** The user's clients with their workload, busiest first. */
  clients: ClientSummary[];
  /** Colors a client can take. */
  clientColors: string[];
  /** Delivered tasks, most recent first. */
  delivered: DeliveredItem[];
  /** Pending or delivered, in the Tasks tab. */
  tasksView: TasksView;
  /** The client panel opened to create a client: its name box gets the focus. */
  creatingClient: boolean;
  /** A short-lived mood (happy after capturing) that overrides the snapshot's mood. */
  reactionMood: MoodName | null;
  /** The latest reminder: shown in the notch, then above the Today list until seen. */
  alert: TikiAlert | null;
  /** A reminder arrived and the island has not been opened since: the notch shows a dot. */
  unreadAlert: boolean;
  /** The AI brief of the day, shown above the Today list when there is one. */
  dayBrief: string | null;
  /** The task open in the edit panel. */
  editing: TaskDetails | null;
}
