/** Every tunable value of the interface lives here, nowhere else. */
export const UI_CONFIG = Object.freeze({
  /** How long a reaction mood (happy, curious) shows before going back to the resting mood. */
  reactionMoodDurationMs: 4_000,
  /** How often the island asks the core for a fresh snapshot. */
  snapshotRefreshIntervalMs: 60_000,
  /** Wait after the last keystroke before previewing a quick capture. */
  capturePreviewDelayMs: 250,
  /** Resting the pointer this long on the hidden notch shows the compact island. */
  peekDelayMs: 120,
  /** The compact island folds back this long after the pointer leaves. */
  foldDelayMs: 700,
  mascotSize: Object.freeze({ hidden: 24, compact: 30, card: 72 }),
});

/** Names of the events the desktop shell sends to the interface (see `shell_events.rs`). */
export const SHELL_EVENTS = Object.freeze({
  openCapture: "tiki://open-capture",
  openSettings: "tiki://open-settings",
  toggleIsland: "tiki://toggle-island",
});
