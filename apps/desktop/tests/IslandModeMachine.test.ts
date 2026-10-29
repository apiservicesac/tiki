import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { UI_CONFIG } from "../src/config/uiConfig";
import type { IslandMode } from "../src/island/islandState";
import { IslandModeMachine } from "../src/island/IslandModeMachine";

describe("IslandModeMachine", () => {
  let mode: IslandMode;
  let typing: boolean;
  let machine: IslandModeMachine;

  beforeEach(() => {
    vi.useFakeTimers();
    mode = "hidden";
    typing = false;
    machine = new IslandModeMachine(() => mode, (next) => {
      mode = next;
    }, { typing: () => typing });
  });

  afterEach(() => vi.useRealTimers());

  it("opens in two steps while the pointer rests on the notch", () => {
    machine.send("pointerEnter");
    vi.advanceTimersByTime(UI_CONFIG.peekDelayMs);
    expect(mode).toBe("compact");
    vi.advanceTimersByTime(UI_CONFIG.unfoldDelayMs);
    expect(mode).toBe("expanded");
  });

  it("does not open when the pointer leaves before the delay", () => {
    machine.send("pointerEnter");
    machine.send("pointerLeave");
    vi.advanceTimersByTime(UI_CONFIG.peekDelayMs + UI_CONFIG.unfoldDelayMs);
    expect(mode).toBe("hidden");
  });

  it("folds after the pointer leaves, and stays open if it comes back", () => {
    machine.moveTo("expanded");
    machine.send("pointerLeave");
    machine.send("pointerEnter");
    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(mode).toBe("expanded");

    machine.send("pointerLeave");
    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(mode).toBe("hidden");
  });

  it("does not fold while something holds it open", () => {
    machine.moveTo("expanded");
    typing = true;
    machine.send("pointerLeave");
    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(mode).toBe("expanded");
  });

  it("shows an alert for a while, then folds", () => {
    machine.send("alertArrived");
    expect(mode).toBe("alert");
    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(mode).toBe("hidden");
  });

  it("opens the whole island when the pointer rests on an alert", () => {
    machine.send("alertArrived");
    machine.send("pointerEnter");
    vi.advanceTimersByTime(UI_CONFIG.peekDelayMs);
    expect(mode).toBe("expanded");
    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(mode).toBe("expanded");
  });

  it("drops a waiting move when the island changed mode meanwhile", () => {
    machine.send("alertArrived");
    mode = "expanded";
    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(mode).toBe("expanded");
  });

  it("forgets the waiting move when a mode is chosen", () => {
    machine.send("alertArrived");
    machine.moveTo("alert");
    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(mode).toBe("alert");
  });
});
