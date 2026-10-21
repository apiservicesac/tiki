import type { UserPreferences } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface ReminderSettingsHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
  onSetAutostart: (enabled: boolean) => void;
}

type BriefTimeField = "morningBriefTime" | "eveningBriefTime";

/** Settings for when Tiki speaks up: brief times, notifications, focus mode and autostart. */
export class ReminderSettingsView {
  constructor(private readonly translator: Translator) {}

  render(preferences: UserPreferences, autostart: boolean, handlers: ReminderSettingsHandlers): HTMLElement {
    const onChange = handlers.onChangePreferences;
    return ElementFactory.create("section", {
      className: "settings-group settings-reminders",
      children: [
        ElementFactory.create("h3", { text: this.translator.text("settings.reminders") }),
        ElementFactory.create("div", {
          className: "settings-options",
          children: [
            this.toggle("settings.notifications", preferences.notificationsEnabled, (enabled) =>
              onChange({ ...preferences, notificationsEnabled: enabled })),
            this.toggle("settings.focus_mode", preferences.focusMode, (enabled) =>
              onChange({ ...preferences, focusMode: enabled })),
            this.toggle("settings.autostart", autostart, handlers.onSetAutostart),
          ],
        }),
        this.timeRow("settings.morning_brief", "morningBriefTime", preferences, onChange),
        this.timeRow("settings.evening_brief", "eveningBriefTime", preferences, onChange),
      ],
    });
  }

  private toggle(labelKey: string, enabled: boolean, onToggle: (enabled: boolean) => void): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-option",
      text: this.translator.text(labelKey),
      attributes: { type: "button", role: "switch", "aria-checked": String(enabled), "data-setting": labelKey },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      onToggle(!enabled);
    });
    return button;
  }

  private timeRow(
    labelKey: string,
    field: BriefTimeField,
    preferences: UserPreferences,
    onChange: (preferences: UserPreferences) => void,
  ): HTMLElement {
    const input = ElementFactory.create("input", {
      className: "settings-input settings-time",
      attributes: { type: "time", id: `setting-${field}`, "aria-label": this.translator.text(labelKey) },
    });
    // The core sends "08:30:00"; the picker shows hours and minutes.
    input.value = preferences[field].slice(0, 5);
    input.addEventListener("change", () => {
      if (input.value !== "") {
        onChange({ ...preferences, [field]: input.value });
      }
    });
    return ElementFactory.create("label", {
      className: "settings-row",
      children: [ElementFactory.create("span", { className: "settings-row-label", text: this.translator.text(labelKey) }), input],
    });
  }
}
