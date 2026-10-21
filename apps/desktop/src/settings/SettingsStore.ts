import type { UserPreferences } from "../backend/contracts";
import type { AiSettingsState } from "./AiSettingsView";
import type { UpdateState } from "./UpdateSettingsView";

/** What the advanced settings show besides the preferences themselves. */
export interface SettingsState {
  aiSettings: AiSettingsState;
  /** Whether Tiki starts with Windows; the shell owns this setting. */
  autostart: boolean;
  updates: UpdateState;
}

/** Read and change the settings state; every change redraws the window. */
export interface SettingsStore {
  state(): Readonly<SettingsState>;
  preferences(): Readonly<UserPreferences>;
  update(changes: Partial<SettingsState>): void;
}

const initialSettingsState: SettingsState = {
  aiSettings: { configured: { anthropic: false, openai: false }, models: [] },
  autostart: false,
  updates: { currentVersion: "", status: "idle", availableVersion: null },
};

export const INITIAL_SETTINGS_STATE: Readonly<SettingsState> = Object.freeze(initialSettingsState);
