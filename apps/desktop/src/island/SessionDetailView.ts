import type { SessionStatus, TranscriptSegment } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { DurationText } from "../dom/DurationText";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { OpenSession } from "./islandState";

export interface SessionDetailHandlers {
  onDeleteSession: (sessionId: number) => void;
  onRetryTranscription: (sessionId: number) => void;
}

/** A transcript line on screen, with the stretch of the media it covers. */
interface ShownLine {
  startMs: number;
  element: HTMLElement;
}

/**
 * One recorded session: its player, the transcript labelled "You" and "Others" that follows
 * the player (the line being said is highlighted, and clicking a line plays from there), and
 * deleting it. The player is kept between redraws, so a refresh never stops playback.
 */
export class SessionDetailView {
  private player: { sessionId: number; element: HTMLMediaElement } | null = null;
  private lines: ShownLine[] = [];
  /** The search hit the player already started at, so a redraw does not jump back. */
  private startedAt: { sessionId: number; startAtMs: number } | null = null;
  private confirmingDelete = false;

  constructor(private readonly translator: Translator) {}

  render(session: OpenSession, handlers: SessionDetailHandlers): HTMLElement {
    const { summary, transcript } = session.details;
    const player = session.mediaUrl === null ? null : this.playerOf(summary.sessionId, summary.mode === "audio_and_video", session.mediaUrl);
    this.lines = [];
    const transcriptList = ElementFactory.create("div", {
      className: "session-transcript",
      children: transcript.length > 0
        ? transcript.map((segment) => this.line(segment))
        : [ElementFactory.create("p", { className: "island-empty", text: this.statusNote(summary.status) })],
    });
    this.startAt(session);
    this.highlight();
    return ElementFactory.create("section", {
      className: "session-detail",
      attributes: { "data-session-id": String(summary.sessionId) },
      children: [
        ElementFactory.create("div", {
          className: "session-detail-head",
          children: [
            ElementFactory.create("strong", { text: summary.title }),
            ElementFactory.create("small", {
              text: [summary.startedLabel, DurationText.clock(summary.recordedMs), this.translator.text(`session.status.${summary.status}`)]
                .join(UI_CONFIG.textMarks.separator),
            }),
          ],
        }),
        player ?? ElementFactory.create("p", { className: "session-note", text: this.translator.text("session.media_removed") }),
        transcriptList,
        this.actions(summary.sessionId, summary.status, handlers),
      ],
    });
  }

  /** Forgets the player, so a closed session stops playing. */
  reset(): void {
    this.player?.element.pause();
    this.player = null;
    this.startedAt = null;
    this.confirmingDelete = false;
  }

  /** The session's player, the same element while the same session is open. */
  private playerOf(sessionId: number, video: boolean, url: string): HTMLMediaElement {
    if (this.player?.sessionId === sessionId) {
      return this.player.element;
    }
    this.reset();
    const element = ElementFactory.create(video ? "video" : "audio", {
      className: "session-player",
      attributes: { controls: "", preload: "metadata", src: url },
    });
    element.addEventListener("timeupdate", () => this.highlight());
    this.player = { sessionId, element };
    return element;
  }

  private line(segment: TranscriptSegment): HTMLElement {
    const element = ElementFactory.textButton({
      className: "session-line",
      attributes: { "data-speaker": segment.speaker, "data-start-ms": String(segment.startMs) },
      children: [
        ElementFactory.create("span", { className: "session-line-time", text: DurationText.clock(segment.startMs) }),
        ElementFactory.create("span", { className: "session-line-speaker", text: this.translator.text(`session.speaker.${segment.speaker}`) }),
        ElementFactory.create("span", { className: "session-line-text", text: segment.text }),
      ],
    }, () => this.play(segment.startMs));
    this.lines.push({ startMs: segment.startMs, element });
    return element;
  }

  /** Plays from `startMs`. */
  private play(startMs: number): void {
    const player = this.player?.element;
    if (!player) {
      return;
    }
    player.currentTime = DurationText.seconds(startMs);
    player.play().catch(() => {
      // The user can still press play; the line stays highlighted.
    });
    this.highlight();
  }

  /** Opened from a search: start at the line found, once. */
  private startAt(session: OpenSession): void {
    if (session.startAtMs === null || !this.player) {
      return;
    }
    const sessionId = session.details.summary.sessionId;
    if (this.startedAt?.sessionId !== sessionId || this.startedAt.startAtMs !== session.startAtMs) {
      this.startedAt = { sessionId, startAtMs: session.startAtMs };
      this.player.element.currentTime = DurationText.seconds(session.startAtMs);
    }
  }

  /** Marks the line being said: the last one that started before the player's time. */
  private highlight(): void {
    const nowMs = DurationText.milliseconds(this.player?.element.currentTime ?? 0);
    const current = this.lines.filter((line) => line.startMs <= nowMs).at(-1);
    this.lines.forEach((line) => line.element.classList.toggle("session-line-now", line === current));
  }

  private statusNote(status: SessionStatus): string {
    return this.translator.text(`session.note.${status}`);
  }

  private actions(sessionId: number, status: SessionStatus, handlers: SessionDetailHandlers): HTMLElement {
    const retry = status === "failed"
      ? [ElementFactory.textButton({ className: "settings-button", text: this.translator.text("session.retry") }, () => handlers.onRetryTranscription(sessionId))]
      : [];
    const remove = ElementFactory.textButton({
      className: "settings-button settings-button-quiet session-delete",
      text: this.translator.text(this.confirmingDelete ? "session.delete_confirm" : "session.delete"),
    }, () => {
      if (this.confirmingDelete) {
        handlers.onDeleteSession(sessionId);
      } else {
        this.confirmingDelete = true;
        remove.textContent = this.translator.text("session.delete_confirm");
      }
    });
    return ElementFactory.create("div", { className: "session-actions", children: [...retry, remove] });
  }
}
