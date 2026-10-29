import type { ActiveRecording } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

/**
 * The red dot and the timer of the session being recorded. The timer keeps counting on its
 * own between snapshots from the core, and stands still while the recording is paused.
 */
export class RecordingIndicator {
  private static readonly MS_PER_SECOND = 1_000;
  private static readonly SECONDS_PER_MINUTE = 60;
  private static readonly SECONDS_PER_HOUR = 3_600;
  private static readonly CLOCK_DIGITS = 2;

  private recording: ActiveRecording | null = null;
  /** When the core measured `recording.recordedMs`, on this page's clock. */
  private measuredAt = 0;
  private timers: HTMLElement[] = [];
  private ticker: ReturnType<typeof setInterval> | null = null;

  constructor(private readonly translator: Translator) {}

  /** The recording of a new snapshot: every timer counts from it. */
  follow(recording: ActiveRecording | null): void {
    this.recording = recording;
    this.measuredAt = Date.now();
    if (recording && !this.ticker) {
      this.ticker = setInterval(() => this.tick(), UI_CONFIG.recordingTimerTickMs);
    } else if (!recording) {
      this.stop();
    }
  }

  /** A dot and a timer; it keeps counting until the recording stops. */
  render(recording: ActiveRecording): HTMLElement {
    const label = this.translator.text(recording.paused ? "recording.paused" : "recording.label");
    const timer = ElementFactory.create("span", { className: "recording-timer", text: this.clockText() });
    this.timers.push(timer);
    return ElementFactory.create("span", {
      className: "recording-indicator",
      attributes: { role: "timer", "aria-label": label, title: label, "data-paused": String(recording.paused) },
      children: [ElementFactory.create("span", { className: "recording-dot", attributes: { "aria-hidden": "true" } }), timer],
    });
  }

  stop(): void {
    if (this.ticker) {
      clearInterval(this.ticker);
      this.ticker = null;
    }
    this.timers = [];
  }

  private tick(): void {
    this.timers = this.timers.filter((timer) => timer.isConnected);
    const text = this.clockText();
    this.timers.forEach((timer) => {
      timer.textContent = text;
    });
  }

  /** "04:07", or "1:02:09" past an hour. */
  private clockText(): string {
    const recording = this.recording;
    if (!recording) {
      return "";
    }
    const counted = recording.paused ? 0 : Date.now() - this.measuredAt;
    const totalSeconds = Math.floor((recording.recordedMs + counted) / RecordingIndicator.MS_PER_SECOND);
    const hours = Math.floor(totalSeconds / RecordingIndicator.SECONDS_PER_HOUR);
    const minutes = Math.floor((totalSeconds % RecordingIndicator.SECONDS_PER_HOUR) / RecordingIndicator.SECONDS_PER_MINUTE);
    const seconds = totalSeconds % RecordingIndicator.SECONDS_PER_MINUTE;
    const pad = (value: number) => String(value).padStart(RecordingIndicator.CLOCK_DIGITS, "0");
    const clock = [pad(minutes), pad(seconds)];
    return (hours > 0 ? [String(hours), ...clock] : clock).join(":");
  }
}
