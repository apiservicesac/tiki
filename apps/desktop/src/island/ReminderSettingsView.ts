import type { UserPreferences } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface ReminderSettingsHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
}

type BriefTimeField = "morningBriefTime" | "eveningBriefTime";

/**
 * Settings for when Tiki speaks up: reminders on or off, the island's chime, Windows
 * notifications as an extra, focus mode and the brief times.
 */
export class ReminderSettingsView {
  constructor(private readonly translator: Translator) {}

  render(preferences: UserPreferences, handlers: ReminderSettingsHandlers): HTMLElement {
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
            this.toggle("settings.alert_sound", preferences.alertSound, (enabled) =>
              onChange({ ...preferences, alertSound: enabled })),
            this.toggle("settings.windows_notifications", preferences.windowsNotifications, (enabled) =>
              onChange({ ...preferences, windowsNotifications: enabled })),
            this.toggle("settings.focus_mode", preferences.focusMode, (enabled) =>
              onChange({ ...preferences, focusMode: enabled })),
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
    input.value = CoreDateTimeText.clock(preferences[field]);
    // The clock icon is hidden; clicking anywhere on the field opens the picker instead.
    input.addEventListener("click", () => input.showPicker?.());
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
