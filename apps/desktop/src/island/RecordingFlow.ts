import type { CaptureTarget, TikiBackend, VideoQuality } from "../backend/contracts";
import type { IslandModeMachine } from "./IslandModeMachine";
import { RESTING_MODES } from "./islandState";
import type { IslandStore } from "./IslandStore";

/**
 * Recording a session from the island: starting it with the mode the user clicked, pausing,
 * resuming and stopping it, and keeping the folded notch in step (with the red dot while a
 * session records).
 */
export class RecordingFlow {
  constructor(
    private readonly backend: TikiBackend,
    private readonly store: IslandStore,
    private readonly modes: IslandModeMachine,
  ) {}

  /**
   * The shortcut: stops the session being recorded, or opens the choice of what to record
   * (nothing records until the user picks a mode).
   */
  async toggle(): Promise<void> {
    if (this.store.state().snapshot.recording) {
      await this.stop();
    } else {
      this.store.update({ mode: "expanded", panel: "record", videoTargets: null });
    }
  }

  /** Records audio only, right away; from Tiki's offer in the notch, the offer goes away. */
  async startAudio(): Promise<void> {
    const fromAlert = this.store.state().mode === "alert";
    if (await this.run(() => this.backend.startRecording("audio_only", null))) {
      this.store.update({ panel: "tabs", alert: null, unreadAlert: false });
      if (fromAlert) {
        this.modes.moveTo("resting");
      }
    }
  }

  /** A video session first asks which screen or window to record. */
  async chooseVideo(): Promise<void> {
    try {
      const videoTargets = await this.backend.captureTargets();
      this.store.update({ mode: "expanded", panel: "record", videoTargets, alert: null, unreadAlert: false });
    } catch {
      this.store.react("worried");
    }
  }

  async startVideo(target: CaptureTarget, quality: VideoQuality): Promise<void> {
    const video = { kind: target.kind, id: target.id, quality };
    if (await this.run(() => this.backend.startRecording("audio_and_video", video))) {
      this.store.update({ panel: "tabs", videoTargets: null });
    }
  }

  async pause(): Promise<void> {
    await this.run(() => this.backend.pauseRecording());
  }

  async resume(): Promise<void> {
    await this.run(() => this.backend.resumeRecording());
  }

  async stop(): Promise<void> {
    if (await this.run(() => this.backend.stopRecording())) {
      this.store.react("happy");
    }
  }

  /** A new snapshot came: the folded notch shows the red dot exactly while a session records. */
  followSnapshot(): void {
    const state = this.store.state();
    if (RESTING_MODES.includes(state.mode) && state.mode !== this.modes.restingMode) {
      this.modes.send(state.snapshot.recording ? "recordingStarted" : "recordingStopped");
    }
  }

  /** Asks the shell for a change, then shows the result; a refusal shows a worried face. */
  private async run(change: () => Promise<void>): Promise<boolean> {
    let done = true;
    try {
      await change();
    } catch {
      done = false;
      this.store.react("worried");
    }
    await this.store.refresh();
    return done;
  }
}
