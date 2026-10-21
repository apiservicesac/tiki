/**
 * Shapes the Rust core sends to the interface (see `tiki_core`).
 * Field names are camelCase because the core serializes them that way.
 */
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

export type TaskStatus = "to_do" | "in_progress" | "testing" | "waiting_on_others" | "done";

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
  /** Tells the shell which part of the window is the island; the rest lets clicks through. */
  setHitArea(area: HitArea): Promise<void>;
  /** Calls `handler` whenever the shell emits `eventName`; returns a function that stops listening. */
  listen(eventName: string, handler: () => void): Promise<() => void>;
}
