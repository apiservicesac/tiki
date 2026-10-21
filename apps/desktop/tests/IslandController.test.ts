import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SHELL_EVENTS, UI_CONFIG } from "../src/config/uiConfig";
import type { IslandController } from "../src/island/IslandController";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";

describe("IslandController", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let controller: IslandController;

  const notch = () => root.querySelector<HTMLElement>(".island-notch");

  beforeEach(async () => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend();
    controller = await new TikiApp(root, backend).start();
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

  it("opens without a click: compact first, then the whole island", () => {
    vi.useFakeTimers();
    notch()?.dispatchEvent(new Event("pointerenter"));

    vi.advanceTimersByTime(UI_CONFIG.peekDelayMs);
    expect(notch()?.dataset.mode).toBe("compact");
    expect(root.querySelector(".notch-summary")?.textContent).toBe("3 hoy · 2 esperando");

    vi.advanceTimersByTime(UI_CONFIG.unfoldDelayMs);
    expect(notch()?.dataset.mode).toBe("expanded");
  });

  it("folds back when the pointer leaves the task list", () => {
    vi.useFakeTimers();
    controller.setMode("expanded");
    notch()?.dispatchEvent(new Event("pointerleave"));

    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(notch()?.dataset.mode).toBe("hidden");
  });

  it("stays open while the user is typing a capture", () => {
    vi.useFakeTimers();
    controller.openPanel("capture");
    notch()?.dispatchEvent(new Event("pointerleave"));

    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(notch()?.dataset.mode).toBe("expanded");
  });

  it("keeps the same notch element so its size can animate", () => {
    const firstNotch = notch();
    controller.setMode("expanded");
    expect(notch()).toBe(firstNotch);
  });

  it("tells the shell where the island is so other clicks pass through", () => {
    expect(backend.hitAreas.length).toBeGreaterThan(0);
  });

  it("keeps what the user types when the island refreshes", async () => {
    controller.openPanel("capture");
    const input = root.querySelector<HTMLInputElement>("#capture-input");
    if (input) {
      input.value = "llamar a altamira";
    }
    await controller.refresh();
    expect(root.querySelector<HTMLInputElement>("#capture-input")?.value).toBe("llamar a altamira");
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
  });

  it("switches language in place, keeping Settings open", async () => {
    controller.openPanel("settings");
    await controller.changePreferences({ locale: "english", islandPlacement: "top" });

    expect(controller.currentState.mode).toBe("expanded");
    expect(controller.currentState.panel).toBe("settings");
    expect(root.querySelector(".island-head-title strong")?.textContent).toBe("Settings");
    expect(document.documentElement.lang).toBe("en");
  });

  it("switches tabs instantly, without fading the card", () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-tab="agenda"]')?.click();
    expect(root.querySelector(".island-content-leaving")).toBeNull();
  });
});
