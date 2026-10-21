/**
 * Shapes the Rust core sends to the interface (see `tiki_core`).
 * Field names are camelCase because the core serializes them that way.
 */
import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../ai";
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

export type Urgency = "critical" | "soon" | "waiting" | "normal";

export const TASK_STATUSES = Object.freeze(["to_do", "in_progress", "testing", "waiting_on_others", "done"] as const);
export type TaskStatus = (typeof TASK_STATUSES)[number];

export const PRIORITIES = Object.freeze(["low", "normal", "high", "urgent"] as const);
export type Priority = (typeof PRIORITIES)[number];

export const ISLAND_PLACEMENTS = Object.freeze(["top", "bottom", "left", "right"] as const);
export type IslandPlacement = (typeof ISLAND_PLACEMENTS)[number];

export const LOCALE_NAMES = Object.freeze(["spanish", "english"] as const);
export type LocaleName = (typeof LOCALE_NAMES)[number];

export interface SnapshotItem {
  taskId: number;
  title: string;
  clientName: string | null;
  clientColor: string | null;
  detailLabel: string;
  urgency: Urgency;
}

export interface SnapshotSection {
  key: string;
  title: string;
  items: SnapshotItem[];
}

export interface IslandSnapshot {
  mood: MoodName;
  moodLabel: string;
  /** Tasks that need attention: overdue, due soon or waiting on others. */
  attentionCount: number;
  /** Overdue or due soon. */
  dueCount: number;
  /** Waiting on others. */
  waitingCount: number;
  closedSummary: string;
  todaySubtitle: string;
  sections: SnapshotSection[];
  narration: string;
}

export interface CaptureDraft {
  title: string;
  clientId: number | null;
  clientName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
}

/** What the core understood from a quick-capture sentence. */
export type CaptureIntent =
  | ({ kind: "task" } & CaptureDraft)
  | { kind: "client"; name: string };

export type CaptureOutcome = { kind: "task" | "client"; id: number };

/** The editable fields of a saved task; the client comes by name. */
export interface TaskEdit {
  title: string;
  notes: string;
  clientName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: Priority;
  status: TaskStatus;
}

export interface TaskDetails extends TaskEdit {
  taskId: number;
}

/** A rectangle in logical pixels, relative to the island window. */
export interface HitArea {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface UserPreferences {
  locale: LocaleName;
  islandPlacement: IslandPlacement;
  aiProvider: AiProviderName;
  /** `null` uses the provider's default model. */
  aiModel: string | null;
  /** "08:30" or "08:30:00". */
  morningBriefTime: string;
  eveningBriefTime: string;
  notificationsEnabled: boolean;
  /** Silences notifications and lets Tiki sleep. */
  focusMode: boolean;
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

/** Everything the interface can ask of the core and the desktop shell. */
export interface TikiBackend {
  islandSnapshot(): Promise<IslandSnapshot>;
  previewCapture(sentence: string): Promise<CaptureIntent>;
  capture(sentence: string): Promise<CaptureOutcome>;
  moveTask(taskId: number, status: TaskStatus): Promise<void>;
  preferences(): Promise<UserPreferences>;
  updatePreferences(preferences: UserPreferences): Promise<void>;
  /** Saves a task exactly as the user confirmed it after the assistant read their sentence. */
  saveConfirmedCapture(capture: InterpretedCapture): Promise<void>;
  clientNames(): Promise<string[]>;
  taskDetails(taskId: number): Promise<TaskDetails>;
  /** Open tasks without ids or notes: what the assistant may read. */
  assistantContext(): Promise<AdvisorContext>;
  editTask(taskId: number, edit: TaskEdit): Promise<void>;
  /** Which providers have a key; the keys themselves never leave the shell. */
  aiStatus(): Promise<AiProviderStatus[]>;
  saveAiKey(provider: AiProviderName, apiKey: string): Promise<void>;
  removeAiKey(provider: AiProviderName): Promise<void>;
  /** The transport the official AI SDKs use: the shell adds the key and sends the request. */
  aiFetch(request: ProxiedRequest): Promise<ProxiedResponse>;
  autostartEnabled(): Promise<boolean>;
  setAutostart(enabled: boolean): Promise<void>;
  /** Tells the shell which part of the window is the island; the rest lets clicks through. */
  setHitArea(area: HitArea): Promise<void>;
  /** Calls `handler` whenever the shell emits `eventName`; returns a function that stops listening. */
  listen(eventName: string, handler: () => void): Promise<() => void>;
}
