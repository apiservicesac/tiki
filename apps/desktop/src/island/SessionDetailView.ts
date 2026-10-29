import type { SessionActionItem, SessionDetails, SessionNotes, SessionStatus, TranscriptSegment } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { UI_CONFIG } from "../config/uiConfig";
import { DurationText } from "../dom/DurationText";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import type { OpenSession } from "./islandState";

export interface SessionDetailHandlers {
  onDeleteSession: (sessionId: number) => void;
  onExportSession: (sessionId: number) => void;
  onRetryTranscription: (sessionId: number) => void;
  onWriteSessionNotes: (sessionId: number) => void;
  onApplySessionTask: (actionItemId: number) => void;
}

/** What the notes section needs to know about the AI. */
export interface SessionNotesState {
  aiReady: boolean;
  /** The session whose notes the AI is writing now. */
  writingNotes: number | null;
}

/** A transcript line on screen, with the stretch of the media it covers. */
interface ShownLine {
  startMs: number;
  element: HTMLElement;
}

/**
 * One recorded session: its player, the AI notes (summary, key points, decisions and the
 * proposed tasks, each created with one click), the transcript labelled "You" and "Others"
 * that follows the player (the line being said is highlighted, and clicking a line plays
 * from there), and exporting or deleting it. The player is kept between redraws, so a refresh never
 * stops playback.
 */
export class SessionDetailView {
  private player: { sessionId: number; element: HTMLMediaElement } | null = null;
  private lines: ShownLine[] = [];
  /** The search hit the player already started at, so a redraw does not jump back. */
  private startedAt: { sessionId: number; startAtMs: number } | null = null;
  private confirmingDelete = false;

  constructor(private readonly translator: Translator) {}

  render(session: OpenSession, ai: SessionNotesState, handlers: SessionDetailHandlers): HTMLElement {
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
        this.notes(session.details, ai, handlers),
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

  /** The AI notes, the AI writing them, a button to write them, or why there are none. */
  private notes(details: SessionDetails, ai: SessionNotesState, handlers: SessionDetailHandlers): HTMLElement {
    const sessionId = details.summary.sessionId;
    const section = (children: HTMLElement[]) => ElementFactory.create("div", { className: "session-notes", children });
    if (details.notes) {
      return section(this.writtenNotes(details.notes, details.actionItems, handlers));
    }
    if (details.summary.status !== "ready" || details.transcript.length === 0) {
      return section([]);
    }
    if (ai.writingNotes === sessionId) {
      return section([ElementFactory.create("p", { className: "session-note", attributes: { role: "status" }, text: this.translator.text("session.notes.writing") })]);
    }
    if (!ai.aiReady) {
      return section([ElementFactory.create("p", { className: "session-note", text: this.translator.text("session.notes.without_ai") })]);
    }
    return section([ElementFactory.textButton({
      className: "settings-button session-write-notes",
      text: this.translator.text("session.notes.write"),
    }, () => handlers.onWriteSessionNotes(sessionId))]);
  }

  private writtenNotes(notes: SessionNotes, items: SessionActionItem[], handlers: SessionDetailHandlers): HTMLElement[] {
    const list = (titleKey: string, lines: string[]) => lines.length === 0 ? [] : [
      ElementFactory.create("h4", { text: this.translator.text(titleKey) }),
      ElementFactory.create("ul", { children: lines.map((line) => ElementFactory.create("li", { text: line })) }),
    ];
    return [
      ElementFactory.create("p", { className: "session-summary", text: notes.summary }),
      ...list("session.notes.key_points", notes.keyPoints),
      ...list("session.notes.decisions", notes.decisions),
      ...(items.length === 0 ? [] : [
        ElementFactory.create("h4", { text: this.translator.text("session.notes.tasks") }),
        ElementFactory.create("div", { className: "session-tasks", children: items.map((item) => this.proposedTask(item, handlers)) }),
      ]),
    ];
  }

  /** A task the AI proposed: created with one click, then marked as created. */
  private proposedTask(item: SessionActionItem, handlers: SessionDetailHandlers): HTMLElement {
    const due = item.dueDate === null
      ? []
      : [ElementFactory.create("small", { text: this.dueLabel(item.dueDate) })];
    const action = item.taskId === null
      ? ElementFactory.textButton({
        className: "settings-button settings-button-quiet session-task-apply",
        text: this.translator.text("session.notes.create_task"),
      }, () => handlers.onApplySessionTask(item.actionItemId))
      : ElementFactory.create("span", {
        className: "session-task-created",
        children: [ElementFactory.fromTrustedMarkup("island-row-check", ICONS.check), document.createTextNode(this.translator.text("session.notes.task_created"))],
      });
    return ElementFactory.create("div", {
      className: "session-task",
      attributes: { "data-action-item-id": String(item.actionItemId) },
      children: [ElementFactory.create("span", { className: "session-task-title", text: item.title }), ...due, action],
    });
  }

  /** "vie 23" / "Fri 23" for a due date the AI read. */
  private dueLabel(isoDate: string): string {
    return new Intl.DateTimeFormat(this.translator.localeCode, { weekday: "short", day: "numeric" })
      .format(CoreDateTimeText.localDay(isoDate));
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
    const exportSession = ElementFactory.textButton({
      className: "settings-button settings-button-quiet session-export",
      text: this.translator.text("session.export"),
    }, () => handlers.onExportSession(sessionId));
    return ElementFactory.create("div", { className: "session-actions", children: [...retry, exportSession, remove] });
  }
}
