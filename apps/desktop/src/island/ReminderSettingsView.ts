import type { UserPreferences } from "../backend/contracts";
import type { Translator } from "../i18n/Translator";
import { SettingRows } from "./SettingRows";

export interface ReminderSettingsHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
}

/**
 * When Tiki speaks up, in three blocks: how alerts arrive (on or off, the chime, Windows
 * notifications), when Tiki keeps quiet (focus mode, the night) and when the two briefs come.
 * Options that depend on alerts being on are dimmed while they are off.
 */
export class ReminderSettingsView {
  private readonly rows: SettingRows;

  constructor(translator: Translator) {
    this.rows = new SettingRows(translator);
  }

  render(preferences: UserPreferences, handlers: ReminderSettingsHandlers): HTMLElement[] {
    return [
      this.alerts(preferences, handlers.onChangePreferences),
      this.silence(preferences, handlers.onChangePreferences),
      this.briefs(preferences, handlers.onChangePreferences),
    ];
  }

  private alerts(preferences: UserPreferences, onChange: (preferences: UserPreferences) => void): HTMLElement {
    const alertsOff = !preferences.notificationsEnabled;
    return this.rows.section("settings.alerts", [
      this.rows.switchRow({ labelKey: "settings.notifications", hintKey: "settings.notifications_hint" },
        preferences.notificationsEnabled, (enabled) => onChange({ ...preferences, notificationsEnabled: enabled })),
      this.rows.switchRow({ labelKey: "settings.alert_sound", hintKey: "settings.alert_sound_hint" },
        preferences.alertSound, (enabled) => onChange({ ...preferences, alertSound: enabled }), alertsOff),
      this.rows.switchRow({ labelKey: "settings.windows_notifications", hintKey: "settings.windows_notifications_hint" },
        preferences.windowsNotifications, (enabled) => onChange({ ...preferences, windowsNotifications: enabled }), alertsOff),
    ]);
  }

  private silence(preferences: UserPreferences, onChange: (preferences: UserPreferences) => void): HTMLElement {
    const quietHours = preferences.quietHours;
    const nightRange = quietHours.enabled
      ? [this.rows.rangeRow(
        { labelKey: "settings.quiet_hours_range", hintKey: "settings.quiet_hours_range_hint" },
        { name: "quietHoursFrom", time: quietHours.from, labelKey: "settings.quiet_hours_from" },
        { name: "quietHoursUntil", time: quietHours.until, labelKey: "settings.quiet_hours_until" },
        (range) => onChange({ ...preferences, quietHours: { ...quietHours, ...range } }),
      )]
      : [];
    return this.rows.section("settings.silence", [
      this.rows.switchRow({ labelKey: "settings.focus_mode", hintKey: "settings.focus_mode_hint" },
        preferences.focusMode, (enabled) => onChange({ ...preferences, focusMode: enabled })),
      this.rows.switchRow({ labelKey: "settings.quiet_hours", hintKey: "settings.quiet_hours_hint" },
        quietHours.enabled, (enabled) => onChange({ ...preferences, quietHours: { ...quietHours, enabled } })),
      ...nightRange,
    ]);
  }

  private briefs(preferences: UserPreferences, onChange: (preferences: UserPreferences) => void): HTMLElement {
    return this.rows.section("settings.briefs", [
      this.rows.timeRow({ labelKey: "settings.morning_brief", hintKey: "settings.morning_brief_hint" },
        "morningBriefTime", preferences.morningBriefTime, (time) => onChange({ ...preferences, morningBriefTime: time })),
      this.rows.timeRow({ labelKey: "settings.evening_brief", hintKey: "settings.evening_brief_hint" },
        "eveningBriefTime", preferences.eveningBriefTime, (time) => onChange({ ...preferences, eveningBriefTime: time })),
    ]);
  }
}
