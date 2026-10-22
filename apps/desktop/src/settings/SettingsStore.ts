import type { CalendarSettingsState } from "../advanced/CalendarAccountsView";
import type { UserPreferences } from "../backend/contracts";
import type { AiSettingsState } from "./AiSettingsView";
import type { UpdateState } from "./UpdateSettingsView";

/** What the advanced settings show besides the preferences themselves. */
export interface SettingsState {
  aiSettings: AiSettingsState;
  /** Whether Tiki starts with Windows; the shell owns this setting. */
  autostart: boolean;
  updates: UpdateState;
  calendars: CalendarSettingsState;
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
  calendars: { status: { accounts: [], available: [] }, connecting: null, failed: false },
};

export const INITIAL_SETTINGS_STATE: Readonly<SettingsState> = Object.freeze(initialSettingsState);
