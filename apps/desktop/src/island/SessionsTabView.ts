import type { ModelDownload, SessionSearchHit, SessionStorage, SessionSummary } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ByteSizeText } from "../dom/ByteSizeText";
import { DurationText } from "../dom/DurationText";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import { ItemRow } from "./ItemRow";

export interface SessionsTabData {
  sessions: SessionSummary[];
  hits: SessionSearchHit[] | null;
  storage: SessionStorage | null;
  modelDownload: ModelDownload | null;
}

export interface SessionsTabHandlers {
  onOpenSession: (sessionId: number, startAtMs: number | null) => void;
  onSearchSessions: (text: string) => void;
  onRetryTranscription: (sessionId: number) => void;
}

/**
 * The Sessions tab: every recorded session (newest first) with its status, a search over
 * the transcripts that opens a session at the line found, the model download while it lasts
 * and what the media takes on disk. Built once and kept, so the search survives refreshes.
 */
export class SessionsTabView {
  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private download: HTMLElement | null = null;
  private list: HTMLElement | null = null;
  private storage: HTMLElement | null = null;
  private handlers: SessionsTabHandlers | null = null;
  private readonly itemRow: ItemRow;

  constructor(private readonly translator: Translator) {
    this.itemRow = new ItemRow(translator);
  }

  render(data: SessionsTabData, handlers: SessionsTabHandlers): HTMLElement {
    this.handlers = handlers;
    this.element ??= this.build();
    this.redraw(data);
    return this.element;
  }

  /** Forgets the tab, so it is rebuilt in a new language. */
  reset(): void {
    this.element = null;
  }

  private build(): HTMLElement {
    this.search = ElementFactory.create("input", {
      className: "settings-input tasks-search",
      attributes: {
        type: "search",
        id: "sessions-search",
        autocomplete: "off",
        placeholder: this.translator.text("sessions.search"),
        "aria-label": this.translator.text("sessions.search"),
      },
    });
    this.search.addEventListener("input", () => this.handlers?.onSearchSessions(this.search?.value ?? ""));
    this.download = ElementFactory.create("p", { className: "sessions-download", attributes: { role: "status" } });
    this.list = ElementFactory.create("div", { className: "island-list sessions-list" });
    this.storage = ElementFactory.create("p", { className: "sessions-storage" });
    return ElementFactory.create("div", {
      className: "sessions-tab",
      children: [
        ElementFactory.create("label", {
          className: "tasks-search-box",
          children: [ElementFactory.fromTrustedMarkup("tasks-search-icon", ICONS.search), this.search],
        }),
        this.download,
        this.list,
        this.storage,
      ],
    });
  }

  private redraw(data: SessionsTabData): void {
    const sizes = new ByteSizeText(this.translator.localeCode);
    if (this.download) {
      this.download.hidden = data.modelDownload === null;
      this.download.textContent = data.modelDownload
        ? this.translator.format("sessions.model_download", {
          share: sizes.share(data.modelDownload.downloadedBytes, data.modelDownload.totalBytes),
        })
        : "";
    }
    if (this.storage) {
      this.storage.textContent = data.storage
        ? this.translator.format("sessions.storage", {
          size: sizes.megabytes(data.storage.mediaBytes),
          days: data.storage.mediaDays,
        })
        : "";
    }
    this.list?.replaceChildren(...(data.hits === null ? this.sessionRows(data.sessions) : this.hitRows(data.hits)));
  }

  private sessionRows(sessions: SessionSummary[]): HTMLElement[] {
    if (sessions.length === 0) {
      return [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("sessions.empty") })];
    }
    return sessions.map((session) => this.itemRow.render({
      variant: "session-row",
      attributes: { "data-session-id": String(session.sessionId) },
      lead: this.itemRow.kindIcon(session.mode === "audio_and_video" ? ICONS.video : ICONS.record),
      title: session.title,
      context: [session.startedLabel, DurationText.clock(session.recordedMs)].join(UI_CONFIG.textMarks.separator),
      onOpen: () => this.handlers?.onOpenSession(session.sessionId, null),
      badge: this.itemRow.when(UI_CONFIG.sessionStatusTones[session.status], this.translator.text(`session.status.${session.status}`)),
      trailing: session.status === "failed" ? this.retryButton(session.sessionId) : null,
    }));
  }

  private hitRows(hits: SessionSearchHit[]): HTMLElement[] {
    if (hits.length === 0) {
      return [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("sessions.no_match") })];
    }
    return hits.map((hit) => this.itemRow.render({
      variant: "session-row session-hit",
      attributes: { "data-session-id": String(hit.sessionId) },
      lead: this.itemRow.time(DurationText.clock(hit.startMs)),
      title: hit.text,
      context: hit.sessionTitle,
      onOpen: () => this.handlers?.onOpenSession(hit.sessionId, hit.startMs),
      badge: this.itemRow.when("normal", this.translator.text(`session.speaker.${hit.speaker}`)),
      trailing: null,
    }));
  }

  private retryButton(sessionId: number): HTMLElement {
    return ElementFactory.textButton({
      className: "settings-button settings-button-quiet session-retry",
      text: this.translator.text("session.retry"),
    }, () => this.handlers?.onRetryTranscription(sessionId));
  }
}
