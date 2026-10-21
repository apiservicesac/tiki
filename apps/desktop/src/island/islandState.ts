import type { IslandPlacement, IslandSnapshot, MoodName, TaskDetails } from "../backend/contracts";
import type { AiSettingsState } from "./AiSettingsView";
import type { UpdateState } from "./UpdateSettingsView";

/** How much of the island is visible, like the notch of dooai. */
export type IslandMode = "hidden" | "compact" | "expanded";

/** What the expanded island shows in its card. */
export type IslandPanel = "tasks" | "capture" | "settings" | "edit" | "ask";

export type IslandTab = "today" | "tickets" | "agenda";

/** Tabs in display order, each with its translation key. */
export const ISLAND_TABS: ReadonlyArray<{ tab: IslandTab; labelKey: string }> = Object.freeze([
  { tab: "today", labelKey: "island.tab.today" },
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
  /** A short-lived mood (happy after capturing) that overrides the snapshot's mood. */
  reactionMood: MoodName | null;
  aiSettings: AiSettingsState;
  /** Whether Tiki starts with Windows; the shell owns this setting. */
  autostart: boolean;
  updates: UpdateState;
  /** The AI brief of the day, shown above the Today list when there is one. */
  dayBrief: string | null;
  /** The task open in the edit panel, with the clients it can be assigned to. */
  editing: { task: TaskDetails; clientNames: string[] } | null;
}
