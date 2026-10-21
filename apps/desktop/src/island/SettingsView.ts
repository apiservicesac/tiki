import { ISLAND_PLACEMENTS, LOCALE_NAMES, type IslandPlacement, type UserPreferences } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { AiSettingsView, type AiSettingsHandlers, type AiSettingsState } from "./AiSettingsView";
import { ReminderSettingsView, type ReminderSettingsHandlers } from "./ReminderSettingsView";
import { UpdateSettingsView, type UpdateSettingsHandlers, type UpdateState } from "./UpdateSettingsView";
import type { Translator } from "../i18n/Translator";

export type SettingsHandlers = AiSettingsHandlers & ReminderSettingsHandlers & UpdateSettingsHandlers;

/** What the settings card shows besides the preferences themselves. */
export interface SettingsState {
  ai: AiSettingsState;
  autostart: boolean;
  updates: UpdateState;
}

/**
 * Settings card: where the island hangs, which language Tiki speaks, when it reminds the
 * user and its AI assistant.
 */
export class SettingsView {
  private readonly aiSettings: AiSettingsView;
  private readonly reminderSettings: ReminderSettingsView;
  private readonly updateSettings: UpdateSettingsView;

  constructor(private readonly translator: Translator) {
    this.aiSettings = new AiSettingsView(translator);
    this.reminderSettings = new ReminderSettingsView(translator);
    this.updateSettings = new UpdateSettingsView(translator);
  }

  render(preferences: UserPreferences, state: SettingsState, handlers: SettingsHandlers): HTMLElement {
    const onChange = handlers.onChangePreferences;
    return ElementFactory.create("div", {
      className: "settings",
      children: [
        this.group(
          "settings.placement",
          ISLAND_PLACEMENTS.map((placement) =>
            this.option(
              `placement.${placement}`,
              preferences.islandPlacement === placement,
              () => onChange({ ...preferences, islandPlacement: placement }),
              this.placementPreview(placement),
            )),
        ),
        this.group(
          "settings.language",
          LOCALE_NAMES.map((locale) =>
            this.option(
              `language.${locale}`,
              preferences.locale === locale,
              () => onChange({ ...preferences, locale }),
            )),
        ),
        this.reminderSettings.render(preferences, state.autostart, handlers),
        this.aiSettings.render(preferences, state.ai, handlers),
        this.updateSettings.render(state.updates, handlers),
      ],
    });
  }

  private group(titleKey: string, options: HTMLElement[]): HTMLElement {
    return ElementFactory.create("section", {
      className: "settings-group",
      children: [
        ElementFactory.create("h3", { text: this.translator.text(titleKey) }),
        ElementFactory.create("div", { className: "settings-options", attributes: { role: "radiogroup" }, children: options }),
      ],
    });
  }

  private option(labelKey: string, selected: boolean, onSelect: () => void, preview?: HTMLElement): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-option",
      attributes: { type: "button", role: "radio", "aria-checked": String(selected) },
      children: [
        ...(preview ? [preview] : []),
        ElementFactory.create("span", { text: this.translator.text(labelKey) }),
      ],
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      onSelect();
    });
    return button;
  }

  /** A tiny screen with the island drawn on the chosen edge. */
  private placementPreview(placement: IslandPlacement): HTMLElement {
    return ElementFactory.create("span", {
      className: "settings-screen",
      children: [ElementFactory.create("span", { className: `settings-screen-island settings-screen-${placement}` })],
    });
  }
}
