import type { RecordingMode, TikiBackend } from "../backend/contracts";
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
      this.store.update({ mode: "expanded", panel: "record" });
    }
  }

  async start(mode: RecordingMode): Promise<void> {
    if (await this.run(() => this.backend.startRecording(mode))) {
      this.store.update({ panel: "tabs" });
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
