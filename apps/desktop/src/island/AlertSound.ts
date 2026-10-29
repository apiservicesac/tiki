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
    UI_CONFIG.alertChime.notesHz.forEach((frequency, index) =>
      this.note(context, frequency, index * UI_CONFIG.alertChime.noteSeconds));
  }

  private note(context: AudioContext, frequency: number, delay: number): void {
    const { noteSeconds, volume, attackSeconds, fadeFloor, fadeNotes, stopNotes } = UI_CONFIG.alertChime;
    const start = context.currentTime + delay;
    const oscillator = context.createOscillator();
    const gain = context.createGain();
    oscillator.type = "sine";
    oscillator.frequency.value = frequency;
    gain.gain.setValueAtTime(0, start);
    gain.gain.linearRampToValueAtTime(volume, start + attackSeconds);
    gain.gain.exponentialRampToValueAtTime(fadeFloor, start + noteSeconds * fadeNotes);
    oscillator.connect(gain).connect(context.destination);
    oscillator.start(start);
    oscillator.stop(start + noteSeconds * stopNotes);
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
