import { TRANSCRIPTION_QUALITIES, type UserPreferences } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import type { Translator } from "../i18n/Translator";
import { SettingRows } from "../settings/SettingRows";

export interface SessionsSectionHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
}

/** Recorded sessions: the transcription model, the automatic AI notes and how long media is kept. */
export class SessionsSectionView {
  private readonly rows: SettingRows;

  constructor(private readonly translator: Translator) {
    this.rows = new SettingRows(translator);
  }

  render(preferences: UserPreferences, handlers: SessionsSectionHandlers): HTMLElement[] {
    const change = (changes: Partial<UserPreferences>) => handlers.onChangePreferences({ ...preferences, ...changes });
    return [
      this.rows.section("advanced.sessions.recordings", [
        this.rows.choiceRow(
          { labelKey: "advanced.sessions.quality", hintKey: "advanced.sessions.quality_hint" },
          TRANSCRIPTION_QUALITIES.map((quality) => ({ value: quality, label: this.translator.text(`transcription.quality.${quality}`) })),
          preferences.transcriptionQuality,
          (quality) => change({ transcriptionQuality: quality }),
        ),
        this.rows.switchRow(
          { labelKey: "advanced.sessions.notes_automatic", hintKey: "advanced.sessions.notes_hint" },
          preferences.sessionNotesAutomatic,
          (enabled) => change({ sessionNotesAutomatic: enabled }),
        ),
        this.rows.choiceRow(
          { labelKey: "advanced.sessions.media", hintKey: "advanced.sessions.media_hint" },
          UI_CONFIG.sessionMediaDayChoices.map((days) => ({ value: days, label: this.translator.format("advanced.sessions.days", { days }) })),
          preferences.sessionMediaDays,
          (days) => change({ sessionMediaDays: days }),
        ),
      ]),
    ];
  }
}
