import type { TikiAssistant } from "../assistant/TikiAssistant";
import { LOCALE_CODES, type TikiBackend, type UserPreferences } from "../backend/contracts";
import type { CaptureView } from "../capture/CaptureView";
import type { Translator } from "../i18n/Translator";
import type { IslandStore } from "./IslandStore";

/** The user's preferences: saved from the island or the advanced window, applied in place. */
export class PreferencesFlow {
  constructor(
    private preferences: UserPreferences,
    private readonly backend: TikiBackend,
    private readonly translator: Translator,
    private readonly captureView: CaptureView,
    private readonly assistant: TikiAssistant,
    private readonly store: IslandStore,
  ) {}

  get current(): Readonly<UserPreferences> {
    return this.preferences;
  }

  /** Saves the preferences and applies them, without closing the island. */
  async change(preferences: UserPreferences): Promise<void> {
    await this.backend.updatePreferences(preferences);
    await this.apply(preferences);
  }

  /** Preferences or AI keys changed, maybe in the advanced settings window. */
  async reload(): Promise<void> {
    await this.apply(await this.backend.preferences());
    await this.loadAssistantStatus();
  }

  /** Which providers have a key, so Ask Tiki and AI capture know whether they can work. */
  async loadAssistantStatus(): Promise<void> {
    await this.assistant.loadStatus();
    this.store.redraw();
  }

  /** A new language redraws everything in it and drops the brief written in the old one. */
  private async apply(preferences: UserPreferences): Promise<void> {
    const localeChanged = preferences.locale !== this.preferences.locale;
    this.preferences = preferences;
    if (localeChanged) {
      this.translator.use(LOCALE_CODES[preferences.locale]);
      this.captureView.reset();
    }
    this.store.update({ placement: preferences.islandPlacement, dayBrief: localeChanged ? null : this.store.state().dayBrief });
    await this.store.refresh();
  }
}
