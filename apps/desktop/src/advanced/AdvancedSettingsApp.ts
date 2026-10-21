import { TikiAi } from "../ai";
import { TikiAssistant } from "../assistant/TikiAssistant";
import { LOCALE_CODES, type TikiBackend, type UserPreferences } from "../backend/contracts";
import { TauriBackend } from "../backend/TauriBackend";
import { SHELL_EVENTS } from "../config/uiConfig";
import { Translator } from "../i18n/Translator";
import { MascotRenderer } from "../mascot/MascotRenderer";
import { SettingsActions } from "../settings/SettingsActions";
import { INITIAL_SETTINGS_STATE, type SettingsState, type SettingsStore } from "../settings/SettingsStore";
import { ADVANCED_SECTIONS, type AdvancedSection } from "./advancedCatalog";
import { AdvancedSettingsView, type AdvancedSettingsHandlers } from "./AdvancedSettingsView";
import { TauriWindowControls, type WindowControls } from "./WindowControls";

/** Runs the advanced settings window: loads the settings, draws them and saves changes. */
export class AdvancedSettingsApp {
  private state: SettingsState = { ...INITIAL_SETTINGS_STATE };
  private preferences!: UserPreferences;
  private section: AdvancedSection;
  private view!: AdvancedSettingsView;
  private translator!: Translator;
  private actions!: SettingsActions;

  constructor(
    private readonly root: HTMLElement,
    private readonly backend: TikiBackend = new TauriBackend(),
    private readonly windowControls: WindowControls = new TauriWindowControls(),
    initialSection = window.location.hash.slice(1),
  ) {
    this.section = AdvancedSettingsApp.sectionOrDefault(initialSection);
  }

  get currentSection(): AdvancedSection {
    return this.section;
  }

  async start(): Promise<void> {
    this.preferences = await this.backend.preferences();
    this.translator = new Translator(LOCALE_CODES[this.preferences.locale]);
    this.view = new AdvancedSettingsView(this.root, this.translator, new MascotRenderer(), this.windowControls);
    const assistant = new TikiAssistant(this.backend, new TikiAi((request) => this.backend.aiFetch(request)));
    this.actions = new SettingsActions(this.store(), this.backend, assistant);
    this.render();
    await this.backend.listen(SHELL_EVENTS.preferencesChanged, () => void this.reload());
    await this.actions.load();
  }

  selectSection(section: AdvancedSection): void {
    this.section = section;
    this.render();
  }

  async changePreferences(preferences: UserPreferences): Promise<void> {
    const providerChanged = preferences.aiProvider !== this.preferences.aiProvider;
    await this.backend.updatePreferences(preferences);
    this.applyPreferences(preferences);
    if (providerChanged) {
      await this.actions.loadAi();
    }
  }

  /** Something changed in another window, such as the language in the island. */
  private async reload(): Promise<void> {
    this.applyPreferences(await this.backend.preferences());
    await this.actions.loadAi();
  }

  private applyPreferences(preferences: UserPreferences): void {
    this.preferences = preferences;
    this.translator.use(LOCALE_CODES[preferences.locale]);
    this.render();
  }

  private store(): SettingsStore {
    return {
      state: () => this.state,
      preferences: () => this.preferences,
      update: (changes) => {
        this.state = { ...this.state, ...changes };
        this.render();
      },
    };
  }

  private render(): void {
    this.view.render(this.section, this.preferences, this.state, this.handlers());
  }

  private handlers(): AdvancedSettingsHandlers {
    return {
      onSelectSection: (section) => this.selectSection(section),
      onChangePreferences: (preferences) => void this.changePreferences(preferences),
      onSaveKey: (provider, apiKey) => void this.actions.saveAiKey(provider, apiKey),
      onRemoveKey: (provider) => void this.actions.removeAiKey(provider),
      onSetAutostart: (enabled) => void this.actions.setAutostart(enabled),
      onCheckUpdate: () => void this.actions.checkForUpdate(),
      onInstallUpdate: () => void this.actions.installUpdate(),
    };
  }

  private static sectionOrDefault(section: string): AdvancedSection {
    return ADVANCED_SECTIONS.find((known) => known === section) ?? "general";
  }
}
