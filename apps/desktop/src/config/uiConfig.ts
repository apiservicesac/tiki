/** Every tunable value of the interface lives here, nowhere else. */
export const UI_CONFIG = Object.freeze({
  /** How long a reaction mood (happy, curious) shows before going back to the resting mood. */
  reactionMoodDurationMs: 4_000,
  /** How often the island asks the core for a fresh snapshot. */
  snapshotRefreshIntervalMs: 60_000,
  /** Wait after the last keystroke before previewing a quick capture. */
  capturePreviewDelayMs: 250,
  /** Resting the pointer this long on the hidden notch starts opening it. */
  peekDelayMs: 90,
  /**
   * After the compact step, the island unfolds completely after this long: enough for the
   * compact notch to finish opening and rest for a moment, so the two steps read clearly.
   */
  unfoldDelayMs: 800,
  /** A reminder stays in the grown notch this long, then folds leaving an unread dot. */
  alertDurationMs: 12_000,
  /** The island's chime: two notes, each this long, at a gentle volume (0 to 1). */
  alertChime: Object.freeze({ notesHz: Object.freeze([880, 1318.5]), noteSeconds: 0.16, volume: 0.12 }),
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
  mascotSize: Object.freeze({ hidden: 24, compact: 30, alert: 40, card: 72 }),
});

/** Names of the events the desktop shell sends to the interface (see `shell_events.rs`). */
export const SHELL_EVENTS = Object.freeze({
  openCapture: "tiki://open-capture",
  openSettings: "tiki://open-settings",
  toggleIsland: "tiki://toggle-island",
  briefReady: "tiki://brief-ready",
  preferencesChanged: "tiki://preferences-changed",
});
