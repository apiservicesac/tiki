import { RECORDING_MODES, type RecordingMode } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";

export interface RecordingPromptHandlers {
  onStartRecording: (mode: RecordingMode) => void;
}

/**
 * The choice of what to record, reached from the notch button, `Ctrl+Alt+R` or Tiki's offer
 * when a call starts: audio only, or audio and video, always transcribed. It reminds the
 * user to tell the participants; nothing records until a mode is clicked.
 */
export class RecordingPrompt {
  constructor(private readonly translator: Translator) {}

  render(handlers: RecordingPromptHandlers): HTMLElement {
    const options = RECORDING_MODES.map((mode) => ElementFactory.textButton({
      className: "record-option",
      attributes: { "data-mode": mode },
      children: [
        ElementFactory.fromTrustedMarkup("record-option-icon", mode === "audio_only" ? ICONS.record : ICONS.video),
        ElementFactory.create("strong", { text: this.translator.text(`record.mode.${mode}`) }),
        ElementFactory.create("span", { text: this.translator.text(`record.mode.${mode}.hint`) }),
      ],
    }, () => handlers.onStartRecording(mode)));
    return ElementFactory.create("section", {
      className: "record-prompt",
      children: [
        ElementFactory.create("p", { className: "record-consent", text: this.translator.text("record.consent") }),
        ElementFactory.create("div", { className: "record-options", children: options }),
        ElementFactory.create("p", { className: "record-note", text: this.translator.text("record.transcribed") }),
      ],
    });
  }
}
