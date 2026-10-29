import { UI_CONFIG } from "../config/uiConfig";
import { DelayedAction } from "./DelayedAction";
import type { IslandMode } from "./islandState";

/**
 * What moves the island between modes. `arrived` happens right after the machine itself
 * moves the island, so one move can follow another (compact, then expanded).
 */
export type ModeEvent = "pointerEnter" | "pointerLeave" | "alertArrived" | "arrived";

/** Where an event takes the island, and after how long (0 moves it at once). */
export interface ModeTransition {
  to: IslandMode;
  afterMs: number;
  /** Folding waits while something holds the island open, such as typing. */
  holdable?: boolean;
}

/** Reasons that keep the island from folding while they last, by name. */
export type FoldHolds = Readonly<Record<string, () => boolean>>;

/**
 * Every move between modes, by the mode the island is in and the event that happens. An
 * event without an entry only cancels the move waiting to happen. New modes, such as
 * listening or speaking, join as new entries.
 */
export const MODE_TRANSITIONS: Readonly<Record<IslandMode, Readonly<Partial<Record<ModeEvent, ModeTransition>>>>> =
  Object.freeze({
    hidden: {
      pointerEnter: { to: "compact", afterMs: UI_CONFIG.peekDelayMs },
      alertArrived: { to: "alert", afterMs: 0 },
    },
    compact: {
      arrived: { to: "expanded", afterMs: UI_CONFIG.unfoldDelayMs },
      pointerLeave: { to: "hidden", afterMs: UI_CONFIG.foldDelayMs, holdable: true },
      alertArrived: { to: "alert", afterMs: 0 },
    },
    alert: {
      arrived: { to: "hidden", afterMs: UI_CONFIG.alertDurationMs },
      pointerEnter: { to: "expanded", afterMs: UI_CONFIG.peekDelayMs },
      pointerLeave: { to: "hidden", afterMs: UI_CONFIG.foldDelayMs, holdable: true },
      alertArrived: { to: "alert", afterMs: 0 },
    },
    expanded: {
      pointerLeave: { to: "hidden", afterMs: UI_CONFIG.foldDelayMs, holdable: true },
    },
  });

/**
 * Decides when the island grows and folds: resting the pointer opens it in two steps, an
 * alert grows it for a while, leaving folds it unless something holds it open. Only one
 * move waits at a time, and it is dropped if the island changed mode meanwhile.
 */
export class IslandModeMachine {
  private readonly pending = new DelayedAction();

  constructor(
    private readonly currentMode: () => IslandMode,
    private readonly move: (mode: IslandMode) => void,
    private readonly holds: FoldHolds,
  ) {}

  /** Something happened: moves the island now or later, as the table says. */
  send(event: ModeEvent): void {
    const from = this.currentMode();
    const transition = MODE_TRANSITIONS[from][event];
    this.pending.cancel();
    if (!transition || (transition.holdable && this.isHeld())) {
      return;
    }
    if (transition.afterMs === 0) {
      this.arrive(transition.to);
      return;
    }
    this.pending.schedule(transition.afterMs, () => {
      if (this.currentMode() === from) {
        this.arrive(transition.to);
      }
    });
  }

  /** The user or the app chose a mode: the move waiting to happen no longer does. */
  moveTo(mode: IslandMode): void {
    this.pending.cancel();
    this.move(mode);
  }

  /** Drops the move waiting to happen, such as folding an alert the user opened. */
  cancel(): void {
    this.pending.cancel();
  }

  private isHeld(): boolean {
    return Object.values(this.holds).some((isHolding) => isHolding());
  }

  private arrive(mode: IslandMode): void {
    this.move(mode);
    this.send("arrived");
  }
}
