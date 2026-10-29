/**
 * Shapes the Rust core sends to the interface (see `tiki_core`).
 * Field names are camelCase because the core serializes them that way.
 */
import type { AdvisorContext, AiProviderName, HarnessLesson, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../ai";
import type { LocaleCode } from "../i18n/LocaleCode";

export type MoodName =
  | "calm"
  | "hurry"
  | "worried"
  | "alert"
  | "impatient"
  | "curious"
  | "happy"
  | "sleep";

export type Urgency = "critical" | "soon" | "waiting" | "normal" | "reminder";

/** A task to do, or a standalone reminder that is done once it rings. */
export const ITEM_KINDS = Object.freeze(["task", "reminder"] as const);
export type ItemKind = (typeof ITEM_KINDS)[number];

/** How often a reminder comes back after it rings. */
export const REPEATS = Object.freeze(["never", "daily", "weekdays", "weekly", "monthly"] as const);
export type Repeat = (typeof REPEATS)[number];

/** A correction the user made to an AI capture (see `CaptureLesson` in the core). */
export type CaptureLesson = HarnessLesson;

export const TASK_STATUSES = Object.freeze(["to_do", "in_progress", "waiting", "done"] as const);
export type TaskStatus = (typeof TASK_STATUSES)[number];

export const PRIORITIES = Object.freeze(["low", "normal", "high", "urgent"] as const);
export type Priority = (typeof PRIORITIES)[number];

export const ISLAND_PLACEMENTS = Object.freeze(["top", "bottom", "left", "right"] as const);
export type IslandPlacement = (typeof ISLAND_PLACEMENTS)[number];

export const LOCALE_NAMES = Object.freeze(["spanish", "english"] as const);
export type LocaleName = (typeof LOCALE_NAMES)[number];

export interface SnapshotItem {
  taskId: number;
  /** A task, or a standalone reminder: the Tasks tab lists only tasks. */
  kind: ItemKind;
  title: string;
  projectId: number | null;
  projectName: string | null;
  projectColor: string | null;
  detailLabel: string;
  priority: Priority;
  /** Lowercase, without "#". */
  tags: string[];
  /**
   * When it rings (`2026-10-23T11:00:00`): a reminder's moment, or the alarm on a task; and how
   * often a reminder comes back.
   */
  at: string | null;
  repeat: Repeat;
  urgency: Urgency;
}

export interface SnapshotSection {
  /** Stable key: `overdue`, `due_soon`, `reminders` (today's) or `follow_up`. */
  key: string;
  title: string;
  items: SnapshotItem[];
}

export interface IslandSnapshot {
  mood: MoodName;
  moodLabel: string;
  /** Another app holds the microphone: Tiki keeps quiet until the call ends. */
  inCall: boolean;
  /** Tasks that need attention: overdue, due soon or waiting on others. */
  attentionCount: number;
  /** Overdue or due soon. */
  dueCount: number;
  /** Waiting on others. */
  waitingCount: number;
  /** Open work: every open task, without standalone reminders. */
  openCount: number;
  /** Delivered today. */
  deliveredToday: number;
  closedSummary: string;
  todaySubtitle: string;
  /** Today only: overdue, due soon, today's reminders and follow-ups. */
  sections: SnapshotSection[];
  /** Every open item, for the Tasks tab. */
  openItems: SnapshotItem[];
  /** What to do now, and why; `null` when nothing presses. */
  focus: { item: SnapshotItem; reason: string } | null;
  /** Meetings from today on and the items that ring at a time, by day (see `AgendaBuilder`). */
  agenda: AgendaDay[];
  narration: string;
}


/** The editable fields of a saved task; the project comes by name. */
export interface TaskEdit {
  title: string;
  notes: string;
  projectName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: Priority;
  status: TaskStatus;
  /** When to remind, for example `2026-10-23T09:00:00`; `null` for no reminder. */
  remindAt: string | null;
  kind: ItemKind;
  tags: string[];
  repeat: Repeat;
  /** Who or what the item waits for, as the user wrote it. */
  waitingFor: string;
}

export interface TaskDetails extends TaskEdit {
  taskId: number;
}

/**
 * What the user can do with an alert, decided by the core (see `AlertAction`): open what it
 * is about, put a reminder that rang away, snooze the item's reminder, or join the call.
 */
export type AlertAction = "open" | "done" | "snooze" | "join";

/** An alert already written in the user's language (see `TikiAlert` in the core). */
export interface TikiAlert {
  kind: "morning" | "evening" | "task" | "reminder" | "follow_up" | "meeting";
  title: string;
  /** What it is about, in one line: the brief's counts, or the item's or meeting's title. */
  summary: string;
  /** The lines after the summary: what to start with, whom to follow up, an item's project and due day. */
  tips: string[];
  /** In the order to offer them. */
  actions: AlertAction[];
  /** The item an item alert is about. */
  taskId: number | null;
  /** A link to open from the alert: the meeting's video call. */
  link: string | null;
}

/** One line of the Agenda, with the time it shows: "10:00", "10:00–10:30" or "Todo el día". */
export type AgendaEntry =
  | { kind: "meeting"; timeLabel: string; meeting: MeetingItem }
  | { kind: "reminder"; timeLabel: string; item: SnapshotItem }
  | { kind: "task"; timeLabel: string; item: SnapshotItem };

/** The Agenda's lines of one day, earliest first, under its title: "Hoy", "Mañana" or the long date. */
export interface AgendaDay {
  /** ISO date, for example `2026-10-23`. */
  day: string;
  label: string;
  entries: AgendaEntry[];
}

/** Whether an alert chimes and reaches Windows, as the core decided (see `AlertDelivery`). */
export interface AlertDelivery {
  sound: boolean;
  windowsNotification: boolean;
}

/** An alert as the shell delivers it: with how it reaches the user (see `DueAlert`). */
export interface DueAlert extends TikiAlert {
  delivery: AlertDelivery;
}

/** A meeting copied from a connected calendar (see `MeetingItem` in the core). */
export interface MeetingItem {
  title: string;
  /** Local time, for example `2026-10-23T14:30:00`. */
  startsAt: string;
  endsAt: string;
  allDay: boolean;
  location: string;
  joinUrl: string | null;
  accountEmail: string;
}

export const CALENDAR_PROVIDERS = Object.freeze(["microsoft", "google"] as const);
export type CalendarProvider = (typeof CALENDAR_PROVIDERS)[number];

export interface CalendarAccount {
  accountId: number;
  provider: CalendarProvider;
  email: string;
}

/** Connected accounts, and the providers this build can connect. */
export interface CalendarStatus {
  accounts: CalendarAccount[];
  available: CalendarProvider[];
}

/** One project with their workload, busiest first (see `ProjectSummary` in the core). */
export interface ProjectSummary {
  projectId: number;
  name: string;
  color: string;
  openCount: number;
  /** Open tasks overdue or due soon. */
  pressingCount: number;
  /** Context the user keeps for the project; Tiki's AI reads it. */
  notes: string;
}

export interface ProjectChange {
  name: string;
  color: string;
  notes: string;
}

/** One delivered task in the history. */
export interface DeliveredItem {
  taskId: number;
  title: string;
  projectId: number | null;
  projectName: string | null;
  projectColor: string | null;
  /** Lowercase, without "#". */
  tags: string[];
  /** "hoy", "ayer" or a short date. */
  deliveredLabel: string;
}

export interface AvailableUpdate {
  version: string;
  notes: string | null;
}

/** A rectangle in logical pixels, relative to the island window. */
export interface HitArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The night: Tiki sleeps and alerts make no sound (see `QuietHours` in the core). */
export interface QuietHours {
  enabled: boolean;
  /** A time of day, "22:00:00" both ways (see `CoreDateTimeText`); the range may cross midnight. */
  from: string;
  until: string;
}

export interface UserPreferences {
  locale: LocaleName;
  islandPlacement: IslandPlacement;
  aiProvider: AiProviderName;
  /** `null` uses the provider's default model. */
  aiModel: string | null;
  /** A time of day, "08:30:00" both ways (see `CoreDateTimeText`). */
  morningBriefTime: string;
  eveningBriefTime: string;
  /** Master switch for reminders. */
  notificationsEnabled: boolean;
  /** The island plays a chime with each alert. */
  alertSound: boolean;
  /** Also send reminders as Windows notifications. */
  windowsNotifications: boolean;
  /** Silences notifications and lets Tiki sleep. */
  focusMode: boolean;
  quietHours: QuietHours;
}

export const AI_PROVIDERS = Object.freeze(["anthropic", "openai"] as const);

export interface AiProviderStatus {
  provider: AiProviderName;
  configured: boolean;
}

/** Maps the core's locale names to the interface's catalog codes. */
export const LOCALE_CODES: Readonly<Record<LocaleName, LocaleCode>> = Object.freeze({
  spanish: "es",
  english: "en",
});

/** What a refused command sends back (see `CommandError` in the shell): a stable key and a readable message. */
export interface CommandFailure {
  key: string;
  message: string;
}

/** Failures the user can fix in the capture or edit form; each has its own message (`error.<key>`). */
export const USER_ERROR_KEYS = Object.freeze(["reminder_needs_time", "empty_task_title"] as const);

/** Everything the interface can ask of the core and the desktop shell; a refused command rejects with a `BackendError`. */
export interface TikiBackend {
  islandSnapshot(): Promise<IslandSnapshot>;
  moveTask(taskId: number, status: TaskStatus): Promise<void>;
  /** Puts off a reminder for some minutes; one that already rang comes back. */
  snooze(taskId: number, minutes: number): Promise<void>;
  preferences(): Promise<UserPreferences>;
  updatePreferences(preferences: UserPreferences): Promise<void>;
  /** Saves a task exactly as the user confirmed it after the assistant read their sentence. */
  saveConfirmedCapture(capture: InterpretedCapture): Promise<void>;
  projectNames(): Promise<string[]>;
  taskDetails(taskId: number): Promise<TaskDetails>;
  projectSummaries(): Promise<ProjectSummary[]>;
  /** Creates a project; returns its id. */
  createProject(name: string): Promise<number>;
  changeProject(projectId: number, change: ProjectChange): Promise<void>;
  /** Removes a project and sends its items to the trash, where they stay without project. */
  removeProject(projectId: number): Promise<void>;
  projectColors(): Promise<string[]>;
  /** Delivered tasks, most recent first. */
  deliveredTasks(): Promise<DeliveredItem[]>;
  /** The user's recent corrections to AI captures, newest first. */
  captureLessons(): Promise<CaptureLesson[]>;
  recordCaptureLesson(lesson: CaptureLesson): Promise<void>;
  /** Open items and project notes: what the assistant may read. */
  assistantContext(): Promise<AdvisorContext>;
  editTask(taskId: number, edit: TaskEdit): Promise<void>;
  /** Which providers have a key; the keys themselves never leave the shell. */
  aiStatus(): Promise<AiProviderStatus[]>;
  saveAiKey(provider: AiProviderName, apiKey: string): Promise<void>;
  removeAiKey(provider: AiProviderName): Promise<void>;
  /** The transport the official AI SDKs use: the shell adds the key and sends the request. */
  aiFetch(request: ProxiedRequest): Promise<ProxiedResponse>;
  calendarStatus(): Promise<CalendarStatus>;
  /** Signs in in the browser and copies the account's meetings. */
  connectCalendar(provider: CalendarProvider): Promise<CalendarAccount>;
  disconnectCalendar(accountId: number): Promise<void>;
  syncCalendars(): Promise<void>;
  /** Opens a meeting's video call (https only). */
  openMeetingLink(url: string): Promise<void>;
  /** Opens the advanced settings window, optionally on one section. */
  openAdvancedSettings(section?: string): Promise<void>;
  autostartEnabled(): Promise<boolean>;
  appVersion(): Promise<string>;
  /** A newer signed version on GitHub, or `null` when Tiki is up to date. */
  checkForUpdate(): Promise<AvailableUpdate | null>;
  /** Installs the latest version; Tiki restarts when it finishes. */
  installUpdate(): Promise<void>;
  setAutostart(enabled: boolean): Promise<void>;
  /** Tells the shell which part of the window is the island; the rest lets clicks through. */
  setHitArea(area: HitArea): Promise<void>;
  /** Calls `handler` with each alert the shell delivers; returns a function that stops listening. */
  onAlert(handler: (alert: DueAlert) => void): Promise<() => void>;
  /** Calls `handler` whenever the shell emits `eventName`; returns a function that stops listening. */
  listen(eventName: string, handler: () => void): Promise<() => void>;
}
