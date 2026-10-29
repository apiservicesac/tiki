import type {
  ModelDownload,
  SessionDetails,
  SessionSearchHit,
  SessionStorage,
  SessionSummary,
  ProjectSummary,
  DeliveredItem,
  IslandPlacement,
  IslandSnapshot,
  MoodName,
  TaskDetails,
  TikiAlert,
} from "../backend/contracts";

/**
 * How much of the island is visible, like the notch of dooai. `alert` is the notch grown a
 * little to show an alert; `recording` is the resting notch while a session records, with
 * its red dot and timer.
 */
export const ISLAND_MODES = Object.freeze(["hidden", "recording", "compact", "alert", "expanded"] as const);
export type IslandMode = (typeof ISLAND_MODES)[number];

/** The folded notch: the island rests in one of these until something opens it. */
export const RESTING_MODES: readonly IslandMode[] = Object.freeze(["hidden", "recording"]);

/**
 * What the expanded island shows in its card: the tabs, or a panel such as quick capture,
 * a task being edited, the project list, the brief drawn as a receipt, the choice of what
 * to record or a recorded session.
 */
export type IslandPanel = "tabs" | "capture" | "settings" | "edit" | "projects" | "brief" | "record" | "session";

/** Pending tasks, or the delivered history, in the Tasks tab. Reminders live in the Agenda. */
export const TASKS_VIEWS = Object.freeze(["pending", "delivered"] as const);
export type TasksView = (typeof TASKS_VIEWS)[number];

/** Tickets join as a tab when a ticket connector arrives. */
export type IslandTab = "today" | "all_tasks" | "agenda" | "sessions";

/** The session open in its panel: its details, where its media plays from, and where to start. */
export interface OpenSession {
  details: SessionDetails;
  mediaUrl: string | null;
  /** Opened from a search: the player starts at the line found. */
  startAtMs: number | null;
}

/** Everything the island view needs to draw one frame. */
export interface IslandState {
  mode: IslandMode;
  panel: IslandPanel;
  tab: IslandTab;
  placement: IslandPlacement;
  snapshot: IslandSnapshot;
  /** The user's projects with their workload, busiest first. */
  projects: ProjectSummary[];
  /** Colors a project can take. */
  projectColors: string[];
  /** Delivered tasks, most recent first. */
  delivered: DeliveredItem[];
  /** Pending or delivered, in the Tasks tab. */
  tasksView: TasksView;
  /** The project panel opened to create a project: its name box gets the focus. */
  creatingProject: boolean;
  /** A short-lived mood (happy after capturing) that overrides the snapshot's mood. */
  reactionMood: MoodName | null;
  /** The latest alert: shown in the notch, then above the Today list until seen. */
  alert: TikiAlert | null;
  /** An alert arrived and the island has not been opened since: the notch shows a dot. */
  unreadAlert: boolean;
  /** The AI brief of the day, shown in the summary card of Today. */
  dayBrief: string | null;
  /** The AI is writing the brief. */
  preparingBrief: boolean;
  /** The task open in the edit panel. */
  editing: TaskDetails | null;
  /** Recorded sessions, newest first. */
  sessions: SessionSummary[];
  /** What their media takes on disk; `null` until loaded. */
  sessionStorage: SessionStorage | null;
  /** Lines matching the search in the Sessions tab; `null` without a search. */
  sessionHits: SessionSearchHit[] | null;
  openSession: OpenSession | null;
  /** The transcription model downloading; `null` when it is not. */
  modelDownload: ModelDownload | null;
}

/** How the island starts, before it knows where it sits and what the day holds. */
export const ISLAND_STATE_DEFAULTS: Readonly<Omit<IslandState, "placement" | "snapshot">> = Object.freeze({
  mode: "hidden",
  panel: "tabs",
  tab: "today",
  projects: [],
  projectColors: [],
  delivered: [],
  tasksView: "pending",
  creatingProject: false,
  reactionMood: null,
  alert: null,
  unreadAlert: false,
  dayBrief: null,
  preparingBrief: false,
  editing: null,
  sessions: [],
  sessionStorage: null,
  sessionHits: null,
  openSession: null,
  modelDownload: null,
});
