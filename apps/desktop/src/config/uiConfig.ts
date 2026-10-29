import type { MoodName, Priority, SessionStatus, TikiAlert, Urgency } from "../backend/contracts";

/** Every tunable value of the interface lives here, nowhere else. */
export const UI_CONFIG = Object.freeze({
  /** How long a reaction mood (happy, curious) shows before going back to the resting mood. */
  reactionMoodDurationMs: 4_000,
  /** How often the island asks the core for a fresh snapshot. */
  snapshotRefreshIntervalMs: 60_000,
  /** Resting the pointer this long on the hidden notch starts opening it. */
  peekDelayMs: 90,
  /**
   * After the compact step, the island unfolds completely after this long: enough for the
   * compact notch to finish opening and rest for a moment, so the two steps read clearly.
   */
  unfoldDelayMs: 800,
  /** An alert stays in the grown notch this long, then folds leaving an unread dot. */
  alertDurationMs: 12_000,
  /**
   * How each kind of alert presents itself in the notch: the face Tiki makes (and so the
   * color of the eyes, the aura and the countdown) and the translation key of its label.
   */
  alertStyles: Object.freeze({
    reminder: Object.freeze({ mood: "hurry", labelKey: "alert.kind.reminder" }),
    task: Object.freeze({ mood: "hurry", labelKey: "alert.kind.task" }),
    follow_up: Object.freeze({ mood: "impatient", labelKey: "alert.kind.follow_up" }),
    meeting: Object.freeze({ mood: "alert", labelKey: "alert.kind.meeting" }),
    morning: Object.freeze({ mood: "calm", labelKey: "alert.kind.morning" }),
    evening: Object.freeze({ mood: "curious", labelKey: "alert.kind.evening" }),
    recording: Object.freeze({ mood: "curious", labelKey: "alert.kind.recording" }),
  } satisfies Record<TikiAlert["kind"], { mood: MoodName; labelKey: string }>),
  /**
   * The island's chime: two notes, each this long, at a gentle volume (0 to 1). Each note
   * rises quickly, then fades to near silence over a few note lengths, so it sounds like a
   * chime and not a beep.
   */
  alertChime: Object.freeze({
    notesHz: Object.freeze([880, 1318.5]),
    noteSeconds: 0.16,
    volume: 0.12,
    attackSeconds: 0.02,
    /** Exponential fades cannot reach zero: they end at this volume. */
    fadeFloor: 0.0001,
    /** The fade lasts this many note lengths, and the note stops a little after it. */
    fadeNotes: 2.2,
    stopNotes: 2.4,
  }),
  /** The snooze buttons of an alert that can be snoozed: minutes, and the translation key of the label. */
  snoozeMinutes: Object.freeze([
    Object.freeze({ minutes: 10, labelKey: "alert.snooze_10" }),
    Object.freeze({ minutes: 60, labelKey: "alert.snooze_60" }),
    Object.freeze({ minutes: 24 * 60, labelKey: "alert.snooze_tomorrow" }),
  ]),
  /** The island folds back this long after the pointer leaves it. */
  foldDelayMs: 650,
  motion: Object.freeze({
    /** Growing the notch: a long, soft ease-out. */
    openMs: 560,
    /** Shrinking the notch: shorter, so closing never feels slow. */
    closeMs: 360,
    /** The previous content fades out this fast while the new one starts to appear. */
    contentOutMs: 180,
  }),
  /** In the Tasks tab, project groups with more pressing work come first, in this order. */
  urgencyOrder: Object.freeze<Urgency[]>(["critical", "soon", "waiting", "normal", "reminder"]),
  /** Marks set between or before pieces of text; the same in every language. */
  textMarks: Object.freeze({
    /** Between the parts of one line: "Altamira · #billing". */
    separator: " · ",
    /** Between the sentences of one paragraph: the tips of a brief on the receipt. */
    sentences: " ",
    /** Before a title, by priority: "!!" urgent, "!" important; other priorities have none. */
    priority: Object.freeze<Partial<Record<Priority, string>>>({ urgent: "!!", high: "!" }),
  }),
  /** Width of Tiki's face in each place; its height is half of it. */
  faceWidth: Object.freeze({ hidden: 40, compact: 44, alert: 56, card: 76, settingsTitle: 36 }),
  /** The recording timer counts every this long, between snapshots from the core. */
  recordingTimerTickMs: 1_000,
  /** Disk usage is shown in megabytes, as the system's file manager does. */
  bytesPerMegabyte: 1_000_000,
  /** The color of a session's status pill in the Sessions tab, as a time-badge tone. */
  sessionStatusTones: Object.freeze({
    recording: "critical",
    paused: "soon",
    queued: "waiting",
    transcribing: "waiting",
    ready: "done",
    failed: "critical",
  } satisfies Record<SessionStatus, string>),
});

/** Names of the events the desktop shell sends to the interface (see `shell_events.rs`). */
export const SHELL_EVENTS = Object.freeze({
  openCapture: "tiki://open-capture",
  openSettings: "tiki://open-settings",
  toggleIsland: "tiki://toggle-island",
  alertReady: "tiki://alert-ready",
  preferencesChanged: "tiki://preferences-changed",
  calendarSynced: "tiki://calendar-synced",
  inCallChanged: "tiki://in-call-changed",
  recordingChanged: "tiki://recording-changed",
  toggleRecording: "tiki://toggle-recording",
  transcriptReady: "tiki://transcript-ready",
  modelDownloadProgress: "tiki://model-download-progress",
});
