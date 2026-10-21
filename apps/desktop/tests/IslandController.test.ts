import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SHELL_EVENTS, UI_CONFIG } from "../src/config/uiConfig";
import type { IslandController } from "../src/island/IslandController";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";

describe("IslandController", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let controller: IslandController;
  let localeChanges: number;

  const notch = () => root.querySelector<HTMLElement>(".island-notch");

  beforeEach(async () => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend();
    localeChanges = 0;
    controller = await new TikiApp(root, backend, () => localeChanges++).start();
  });

  afterEach(() => {
    controller.stop();
    vi.useRealTimers();
  });

  it("starts as a hidden notch on the saved edge", () => {
    expect(notch()?.dataset.mode).toBe("hidden");
    expect(notch()?.dataset.mood).toBe("worried");
    expect(root.dataset.placement).toBe("top");
  });

  it("peeks into the compact island when the pointer rests on it", () => {
    vi.useFakeTimers();
    notch()?.dispatchEvent(new Event("pointerenter"));
    vi.advanceTimersByTime(UI_CONFIG.peekDelayMs);

    expect(notch()?.dataset.mode).toBe("compact");
    expect(root.querySelector(".notch-summary")?.textContent).toBe("3 hoy · 2 esperando");
  });

  it("expands on click and lists every section of the day", () => {
    notch()?.click();

    const groups = [...root.querySelectorAll(".island-group")].map((group) => group.textContent);
    expect(groups).toEqual(["Atrasado", "Vence pronto", "Te toca", "Esperando a otro"]);
    expect(root.querySelectorAll(".island-row")).toHaveLength(6);
  });

  it("delivers a task with its check button and celebrates", async () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-task-id="1"] .island-row-done')?.click();
    await vi.waitFor(() => expect(backend.movedTasks).toEqual([{ taskId: 1, status: "done" }]));

    await vi.waitFor(() => expect(notch()?.dataset.mood).toBe("happy"));
  });

  it("switches tabs and shows the empty state of Tickets", () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-tab="tickets"]')?.click();

    expect(controller.currentState.tab).toBe("tickets");
    expect(root.querySelector(".island-empty")?.textContent).toContain("Ajustes › Plugins");
  });

  it("opens quick capture when the shell asks for it", () => {
    backend.emit(SHELL_EVENTS.openCapture);
    expect(root.querySelector("#capture-input")).not.toBeNull();
  });

  it("saves a capture, goes back to the list and celebrates", async () => {
    await controller.submitCapture("revisar acta altamira viernes");

    expect(backend.capturedSentences).toEqual(["revisar acta altamira viernes"]);
    expect(controller.currentState.panel).toBe("tasks");
    expect(notch()?.dataset.mood).toBe("happy");
  });

  it("moves the island when another edge is chosen in Settings", async () => {
    backend.emit(SHELL_EVENTS.openSettings);
    root.querySelectorAll<HTMLButtonElement>(".settings-option")[3]?.click();

    await vi.waitFor(() => expect(backend.savedPreferences.islandPlacement).toBe("right"));
    expect(root.dataset.placement).toBe("right");
    expect(localeChanges).toBe(0);
  });

  it("reloads the translations when the language changes", async () => {
    await controller.changePreferences({ locale: "english", islandPlacement: "top" });
    expect(localeChanges).toBe(1);
  });
});
