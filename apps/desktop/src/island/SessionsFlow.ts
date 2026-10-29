import type { ModelDownload, TikiBackend } from "../backend/contracts";
import type { IslandState } from "./islandState";
import type { IslandStore } from "./IslandStore";

/**
 * The Sessions tab's actions: loading the sessions, opening one (at a search hit), searching
 * the transcripts, deleting a session, retrying its transcription and following the model
 * download.
 */
export class SessionsFlow {
  constructor(
    private readonly backend: TikiBackend,
    private readonly store: IslandStore,
  ) {}

  /** The sessions, their disk usage and the open session, fresh from the core. */
  async load(): Promise<Pick<IslandState, "sessions" | "sessionStorage" | "openSession">> {
    const state = this.store.state();
    const open = state.panel === "session" ? state.openSession : null;
    const [sessions, sessionStorage, details] = await Promise.all([
      this.backend.sessions(),
      this.backend.sessionStorage(),
      open ? this.backend.sessionDetails(open.details.summary.sessionId) : Promise.resolve(null),
    ]);
    return {
      sessions,
      sessionStorage,
      openSession: open && details ? { ...open, details, mediaUrl: this.mediaUrl(details.mediaPath) } : null,
    };
  }

  async open(sessionId: number, startAtMs: number | null = null): Promise<void> {
    const details = await this.backend.sessionDetails(sessionId);
    this.store.update({
      mode: "expanded",
      panel: "session",
      openSession: { details, mediaUrl: this.mediaUrl(details.mediaPath), startAtMs },
    });
  }

  close(): void {
    this.store.update({ panel: "tabs", openSession: null });
  }

  async search(text: string): Promise<void> {
    const hits = text.trim() === "" ? null : await this.backend.searchSessions(text);
    this.store.update({ sessionHits: hits });
  }

  async delete(sessionId: number): Promise<void> {
    await this.backend.deleteSession(sessionId);
    this.close();
    await this.store.refresh();
  }

  async retry(sessionId: number): Promise<void> {
    await this.backend.retryTranscription(sessionId);
    await this.store.refresh();
  }

  /** The model downloads the first time the user records; the tab says how far it got. */
  followDownload(progress: ModelDownload): void {
    this.store.update({ modelDownload: progress.downloadedBytes < progress.totalBytes ? progress : null });
  }

  private mediaUrl(mediaPath: string | null): string | null {
    return mediaPath === null ? null : this.backend.mediaUrl(mediaPath);
  }
}
