import type { UserPreferences } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface ReminderSettingsHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
}

/**
 * Settings for when Tiki speaks up: reminders on or off, the island's chime, Windows
 * notifications as an extra, focus mode, the quiet hours of the night and the brief times.
 */
export class ReminderSettingsView {
  constructor(private readonly translator: Translator) {}

  render(preferences: UserPreferences, handlers: ReminderSettingsHandlers): HTMLElement {
    const onChange = handlers.onChangePreferences;
    const quietHours = preferences.quietHours;
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
            this.toggle("settings.quiet_hours", quietHours.enabled, (enabled) =>
              onChange({ ...preferences, quietHours: { ...quietHours, enabled } })),
          ],
        }),
        ...(quietHours.enabled ? this.quietHoursRows(preferences, onChange) : []),
        this.timeRow("settings.morning_brief", "morningBriefTime", preferences.morningBriefTime, (time) =>
          onChange({ ...preferences, morningBriefTime: time })),
        this.timeRow("settings.evening_brief", "eveningBriefTime", preferences.eveningBriefTime, (time) =>
          onChange({ ...preferences, eveningBriefTime: time })),
      ],
    });
  }

  /** When Tiki falls asleep and when it wakes up; shown while the quiet hours are on. */
  private quietHoursRows(preferences: UserPreferences, onChange: (preferences: UserPreferences) => void): HTMLElement[] {
    const quietHours = preferences.quietHours;
    return [
      this.timeRow("settings.quiet_hours_from", "quietHoursFrom", quietHours.from, (from) =>
        onChange({ ...preferences, quietHours: { ...quietHours, from } })),
      this.timeRow("settings.quiet_hours_until", "quietHoursUntil", quietHours.until, (until) =>
        onChange({ ...preferences, quietHours: { ...quietHours, until } })),
    ];
  }

  private toggle(labelKey: string, enabled: boolean, onToggle: (enabled: boolean) => void): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-option",
      text: this.translator.text(labelKey),
      attributes: { role: "switch", "aria-checked": String(enabled), "data-setting": labelKey },
    }, () => onToggle(!enabled));
    return button;
  }

  /** A time picker beside its label; `onPick` receives the time the user chose ("07:45"). */
  private timeRow(labelKey: string, name: string, time: string, onPick: (time: string) => void): HTMLElement {
    const input = ElementFactory.create("input", {
      className: "settings-input settings-time",
      attributes: { type: "time", id: `setting-${name}`, "aria-label": this.translator.text(labelKey) },
    });
    input.value = CoreDateTimeText.clock(time);
    // The clock icon is hidden; clicking anywhere on the field opens the picker instead.
    input.addEventListener("click", () => input.showPicker?.());
    input.addEventListener("change", () => {
      if (input.value !== "") {
        onPick(input.value);
      }
    });
    return ElementFactory.create("label", {
      className: "settings-row",
      children: [ElementFactory.create("span", { className: "settings-row-label", text: this.translator.text(labelKey) }), input],
    });
  }
}
