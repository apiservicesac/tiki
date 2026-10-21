import type { UserPreferences } from "../backend/contracts";
import type { IslandState } from "./islandState";

/** Read and change the island's state; every change redraws it. */
export interface IslandStore {
  state(): Readonly<IslandState>;
  preferences(): Readonly<UserPreferences>;
  update(changes: Partial<IslandState>): void;
}
