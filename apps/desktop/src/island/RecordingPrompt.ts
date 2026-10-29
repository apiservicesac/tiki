import { VIDEO_QUALITIES, type CaptureTarget, type VideoQuality } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";

export interface RecordingPromptHandlers {
  onStartAudioRecording: () => void;
  /** Audio and video: first the screen or window to record. */
  onChooseVideo: () => void;
  onStartVideoRecording: (target: CaptureTarget, quality: VideoQuality) => void;
}

/**
 * The choice of what to record, reached from the notch button, `Ctrl+Alt+R` or Tiki's offer
 * when a call starts: audio only, or audio and video of one screen or window (in normal or
 * high quality), always transcribed. It reminds the user to tell the participants; nothing
 * records until the user clicks.
 */
export class RecordingPrompt {
  /** The quality chosen for video, kept while the prompt is open. */
  private quality: VideoQuality = "normal";

  constructor(private readonly translator: Translator) {}

  render(videoTargets: CaptureTarget[] | null, handlers: RecordingPromptHandlers): HTMLElement {
    return ElementFactory.create("section", {
      className: "record-prompt",
      children: [
        ElementFactory.create("p", { className: "record-consent", text: this.translator.text("record.consent") }),
        videoTargets === null ? this.modes(handlers) : this.videoPicker(videoTargets, handlers),
        ElementFactory.create("p", { className: "record-note", text: this.translator.text("record.transcribed") }),
      ],
    });
  }

  /** Forgets the quality chosen, so the next prompt starts from normal. */
  reset(): void {
    this.quality = "normal";
  }

  private modes(handlers: RecordingPromptHandlers): HTMLElement {
    return ElementFactory.create("div", {
      className: "record-options",
      children: [
        this.modeOption("audio_only", ICONS.record, handlers.onStartAudioRecording),
        this.modeOption("audio_and_video", ICONS.video, handlers.onChooseVideo),
      ],
    });
  }

  private modeOption(mode: "audio_only" | "audio_and_video", icon: string, onPress: () => void): HTMLElement {
    return ElementFactory.textButton({
      className: "record-option",
      attributes: { "data-mode": mode },
      children: [
        ElementFactory.fromTrustedMarkup("record-option-icon", icon),
        ElementFactory.create("strong", { text: this.translator.text(`record.mode.${mode}`) }),
        ElementFactory.create("span", { text: this.translator.text(`record.mode.${mode}.hint`) }),
      ],
    }, onPress);
  }

  /** Which screen or window to record, and in which quality. */
  private videoPicker(targets: CaptureTarget[], handlers: RecordingPromptHandlers): HTMLElement {
    const qualities = VIDEO_QUALITIES.map((quality) => ElementFactory.textButton({
      className: "settings-option",
      text: this.translator.text(`record.quality.${quality}`),
      attributes: { role: "radio", "aria-checked": String(quality === this.quality), "data-quality": quality },
    }, () => {
      this.quality = quality;
      qualities.forEach((option) => option.setAttribute("aria-checked", String(option.dataset.quality === quality)));
    }));
    const choices = targets.map((target) => ElementFactory.textButton({
      className: "record-target",
      attributes: { "data-kind": target.kind, "data-id": String(target.id) },
      children: [
        ElementFactory.fromTrustedMarkup("record-option-icon", target.kind === "screen" ? ICONS.screen : ICONS.window),
        ElementFactory.create("span", { text: this.targetName(target) }),
      ],
    }, () => handlers.onStartVideoRecording(target, this.quality)));
    return ElementFactory.create("div", {
      className: "record-picker",
      children: [
        ElementFactory.create("div", { className: "settings-options", attributes: { role: "radiogroup" }, children: qualities }),
        ElementFactory.create("div", {
          className: "record-targets",
          children: choices.length > 0
            ? choices
            : [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("record.no_targets") })],
        }),
      ],
    });
  }

  private targetName(target: CaptureTarget): string {
    return target.kind === "screen"
      ? this.translator.format("record.screen", { number: target.id })
      : target.name;
  }
}
