import { TRANSCRIPTION_QUALITIES, type UserPreferences } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface SessionsSectionHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
}

/** Recorded sessions: the transcription model, the automatic AI notes and how long media is kept. */
export class SessionsSectionView {
  constructor(private readonly translator: Translator) {}

  render(preferences: UserPreferences, handlers: SessionsSectionHandlers): HTMLElement[] {
    const change = (changes: Partial<UserPreferences>) => handlers.onChangePreferences({ ...preferences, ...changes });
    return [
      this.group("advanced.sessions.quality", "advanced.sessions.quality_hint", TRANSCRIPTION_QUALITIES.map((quality) =>
        this.option(
          this.translator.text(`transcription.quality.${quality}`),
          preferences.transcriptionQuality === quality,
          () => change({ transcriptionQuality: quality }),
          this.translator.text(`transcription.quality.${quality}.hint`),
        ))),
      this.group("advanced.sessions.notes", "advanced.sessions.notes_hint", [
        this.option(
          this.translator.text("advanced.sessions.notes_automatic"),
          preferences.sessionNotesAutomatic,
          () => change({ sessionNotesAutomatic: !preferences.sessionNotesAutomatic }),
        ),
      ]),
      this.group("advanced.sessions.media", "advanced.sessions.media_hint", UI_CONFIG.sessionMediaDayChoices.map((days) =>
        this.option(
          this.translator.format("advanced.sessions.days", { days }),
          preferences.sessionMediaDays === days,
          () => change({ sessionMediaDays: days }),
        ))),
    ];
  }

  private group(titleKey: string, hintKey: string, options: HTMLElement[]): HTMLElement {
    return ElementFactory.create("section", {
      className: "settings-group",
      children: [
        ElementFactory.create("h3", { text: this.translator.text(titleKey) }),
        ElementFactory.create("div", { className: "settings-options", children: options }),
        ElementFactory.create("p", { className: "settings-note", text: this.translator.text(hintKey) }),
      ],
    });
  }

  private option(label: string, checked: boolean, onPress: () => void, title?: string): HTMLElement {
    return ElementFactory.textButton({
      className: "settings-option",
      text: label,
      attributes: { role: "switch", "aria-checked": String(checked), ...(title ? { title } : {}) },
    }, onPress);
  }
}
