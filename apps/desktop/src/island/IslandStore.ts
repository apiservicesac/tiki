import type { MoodName, UserPreferences } from "../backend/contracts";
import type { IslandState } from "./islandState";

/** Read and change the island's state; every change redraws it. */
export interface IslandStore {
  state(): Readonly<IslandState>;
  preferences(): Readonly<UserPreferences>;
  update(changes: Partial<IslandState>): void;
  /** Reloads the day, the projects and the history from the core. */
  refresh(): Promise<void>;
  /** Draws the island again after something outside its state changed, such as the AI keys. */
  redraw(): void;
  /** Shows a reaction mood for a moment (happy after delivering), then the resting mood. */
  react(mood: MoodName): void;
}
