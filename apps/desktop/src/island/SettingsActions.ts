import type { AiProviderName } from "../ai";
import type { TikiAssistant } from "../assistant/TikiAssistant";
import type { TikiBackend } from "../backend/contracts";
import type { IslandStore } from "./IslandStore";
import type { UpdateState } from "./UpdateSettingsView";

/** Loads and changes the settings the shell owns: AI keys, models, start with Windows and updates. */
export class SettingsActions {
  /** Counts AI settings loads, so a slow older load never overwrites a newer one. */
  private aiLoads = 0;

  constructor(
    private readonly store: IslandStore,
    private readonly backend: TikiBackend,
    private readonly assistant: TikiAssistant,
  ) {}

  async load(): Promise<void> {
    await Promise.all([this.loadAi(), this.loadAutostart(), this.loadVersion()]);
  }

  async checkForUpdate(): Promise<void> {
    this.setUpdates({ status: "checking" });
    try {
      const update = await this.backend.checkForUpdate();
      this.setUpdates(update
        ? { status: "available", availableVersion: update.version }
        : { status: "up_to_date", availableVersion: null });
    } catch {
      this.setUpdates({ status: "failed" });
    }
  }

  async installUpdate(): Promise<void> {
    this.setUpdates({ status: "installing" });
    try {
      await this.backend.installUpdate();
    } catch {
      this.setUpdates({ status: "failed" });
    }
  }

  /** Which providers have a key and which models the current one offers. */
  async loadAi(): Promise<void> {
    const load = ++this.aiLoads;
    const configured = await this.assistant.loadStatus();
    const models = await this.assistant.models(this.store.preferences().aiProvider);
    if (load === this.aiLoads) {
      this.store.update({ aiSettings: { configured, models } });
    }
  }

  async saveAiKey(provider: AiProviderName, apiKey: string): Promise<void> {
    await this.backend.saveAiKey(provider, apiKey);
    await this.loadAi();
  }

  async removeAiKey(provider: AiProviderName): Promise<void> {
    await this.backend.removeAiKey(provider);
    await this.loadAi();
  }

  async setAutostart(enabled: boolean): Promise<void> {
    await this.backend.setAutostart(enabled);
    await this.loadAutostart();
  }

  private async loadVersion(): Promise<void> {
    if (this.store.state().updates.currentVersion === "") {
      this.setUpdates({ currentVersion: await this.backend.appVersion() });
    }
  }

  private setUpdates(changes: Partial<UpdateState>): void {
    this.store.update({ updates: { ...this.store.state().updates, ...changes } });
  }

  private async loadAutostart(): Promise<void> {
    try {
      this.store.update({ autostart: await this.backend.autostartEnabled() });
    } catch {
      this.store.update({ autostart: false });
    }
  }
}
