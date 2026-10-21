import { ISLAND_PLACEMENTS, LOCALE_NAMES, type IslandPlacement, type UserPreferences } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { AiSettingsView, type AiSettingsHandlers, type AiSettingsState } from "./AiSettingsView";
import type { Translator } from "../i18n/Translator";

/** Settings card: where the island hangs, which language Tiki speaks and its AI assistant. */
export class SettingsView {
  private readonly aiSettings: AiSettingsView;

  constructor(private readonly translator: Translator) {
    this.aiSettings = new AiSettingsView(translator);
  }

  render(preferences: UserPreferences, aiState: AiSettingsState, handlers: AiSettingsHandlers): HTMLElement {
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
        this.aiSettings.render(preferences, aiState, handlers),
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
