import { ISLAND_PLACEMENTS, LOCALE_NAMES, type IslandPlacement, type UserPreferences } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import { SettingRows } from "../settings/SettingRows";
import { ReminderSettingsView, type ReminderSettingsHandlers } from "./ReminderSettingsView";

export interface SettingsHandlers extends ReminderSettingsHandlers {
  onOpenAdvancedSettings: () => void;
}

/**
 * The island's own settings: where it hangs, which language Tiki speaks and when it reminds
 * the user. Everything else lives in the advanced settings window.
 */
export class SettingsView {
  private readonly reminderSettings: ReminderSettingsView;
  private readonly rows: SettingRows;

  constructor(private readonly translator: Translator) {
    this.reminderSettings = new ReminderSettingsView(translator);
    this.rows = new SettingRows(translator);
  }

  render(preferences: UserPreferences, handlers: SettingsHandlers): HTMLElement {
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
        this.rows.section("settings.notch", [
          this.rows.switchRow({ labelKey: "settings.system_stats", hintKey: "settings.system_stats_hint" },
            preferences.showSystemStats, (enabled) => onChange({ ...preferences, showSystemStats: enabled })),
        ]),
        ...this.reminderSettings.render(preferences, handlers),
        this.advancedLink(handlers.onOpenAdvancedSettings),
      ],
    });
  }

  private advancedLink(onOpen: () => void): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-advanced",
      attributes: { id: "open-advanced-settings" },
      children: [
        ElementFactory.fromTrustedMarkup("settings-advanced-icon", ICONS.sliders),
        ElementFactory.create("span", {
          className: "settings-advanced-text",
          children: [
            ElementFactory.create("strong", { text: this.translator.text("settings.advanced") }),
            ElementFactory.create("small", { text: this.translator.text("settings.advanced_hint") }),
          ],
        }),
        ElementFactory.fromTrustedMarkup("settings-advanced-arrow", ICONS.external),
      ],
    }, onOpen);
    return button;
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
    const button = ElementFactory.textButton({
      className: "settings-option",
      attributes: { role: "radio", "aria-checked": String(selected) },
      children: [
        ...(preview ? [preview] : []),
        ElementFactory.create("span", { text: this.translator.text(labelKey) }),
      ],
    }, onSelect);
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
