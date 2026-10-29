import type { TikiBackend } from "../backend/contracts";
import type { IslandStore } from "../island/IslandStore";
import type { TikiAssistant } from "./TikiAssistant";

/**
 * The AI notes of sessions: written as soon as a transcript is ready (unless the user turned
 * that off, and only with an AI key), or when the user asks; each proposed task becomes a
 * task with one click. Without a key the session keeps its transcript only.
 */
export class SessionNotesFlow {
  constructor(
    private readonly backend: TikiBackend,
    private readonly store: IslandStore,
    private readonly assistant: TikiAssistant,
  ) {}

  /** A transcript is ready: its notes are written now if the user keeps that on. */
  async transcriptReady(sessionId: number): Promise<void> {
    if (this.store.preferences().sessionNotesAutomatic) {
      await this.write(sessionId);
    } else {
      await this.store.refresh();
    }
  }

  /** Writes the session's notes; a failure leaves the transcript, and the user can ask again. */
  async write(sessionId: number): Promise<void> {
    const preferences = this.store.preferences();
    if (!this.assistant.isReady(preferences)) {
      await this.store.refresh();
      return;
    }
    this.store.update({ writingNotes: sessionId });
    try {
      const details = await this.backend.sessionDetails(sessionId);
      if (details.transcript.length > 0) {
        await this.backend.saveSessionNotes(sessionId, await this.assistant.sessionNotes(details, preferences));
      }
    } catch {
      // The transcript is still there; the notes are a bonus.
    } finally {
      this.store.update({ writingNotes: null });
      await this.store.refresh();
    }
  }

  /** Creates the task the AI proposed, linked to its session. */
  async applyTask(actionItemId: number): Promise<void> {
    try {
      await this.backend.applySessionTask(actionItemId);
      this.store.react("happy");
    } catch {
      this.store.react("worried");
    }
    await this.store.refresh();
  }
}
