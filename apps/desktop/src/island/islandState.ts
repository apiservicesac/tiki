import type { IslandPlacement, IslandSnapshot, MoodName } from "../backend/contracts";

/** How much of the island is visible, like the notch of dooai. */
export type IslandMode = "hidden" | "compact" | "expanded";

/** What the expanded island shows in its card. */
export type IslandPanel = "tasks" | "capture" | "settings";

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
}
