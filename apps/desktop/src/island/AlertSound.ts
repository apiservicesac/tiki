import { UI_CONFIG } from "../config/uiConfig";

/**
 * The island's own chime: two soft notes made with Web Audio, so alerts sound the same on
 * every computer and need no audio file. Quietly does nothing where audio is unavailable.
 */
export class AlertSound {
  private context: AudioContext | null = null;

  play(): void {
    const context = this.audioContext();
    if (!context) {
      return;
    }
    void context.resume();
    const { notesHz, noteSeconds, volume } = UI_CONFIG.alertChime;
    notesHz.forEach((frequency, index) => this.note(context, frequency, index * noteSeconds, noteSeconds, volume));
  }

  private note(context: AudioContext, frequency: number, delay: number, seconds: number, volume: number): void {
    const start = context.currentTime + delay;
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.type = "sine";
    oscillator.frequency.value = frequency;
    // A quick attack and a soft fade, so it sounds like a chime and not a beep.
    gain.gain.setValueAtTime(0, start);
    gain.gain.linearRampToValueAtTime(volume, start + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, start + seconds * 2.2);
    oscillator.connect(gain).connect(context.destination);
    oscillator.start(start);
    oscillator.stop(start + seconds * 2.4);
  }

  private audioContext(): AudioContext | null {
    if (this.context) {
      return this.context;
    }
    if (typeof AudioContext === "undefined") {
      return null;
    }
    this.context = new AudioContext();
    return this.context;
  }
}
