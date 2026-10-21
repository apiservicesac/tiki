import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SHELL_EVENTS, UI_CONFIG } from "../src/config/uiConfig";
import type { IslandController } from "../src/island/IslandController";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";
import { FakeShell } from "./support/FakeShell";

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

  it("shows Tiki and the day's summary while resting when work is pending", () => {
    expect(root.querySelector(".notch-hidden-busy .notch-name")?.textContent).toBe("Tiki");
    expect(root.querySelector(".notch-hidden-busy .notch-summary")?.textContent).toBe("3 hoy · 2 esperando");
  });

  it("lays the island out vertically on the left and right edges", async () => {
    await controller.changePreferences({ ...backend.savedPreferences, islandPlacement: "left" });
    expect(root.dataset.orientation).toBe("vertical");
    const bubbles = [...root.querySelectorAll(".notch-count")].map((bubble) => bubble.textContent);
    expect(bubbles).toEqual(["3", "2"]);

    await controller.changePreferences({ ...backend.savedPreferences, islandPlacement: "bottom" });
    expect(root.dataset.orientation).toBe("horizontal");
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

  it("opens a task for editing and saves the changes", async () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-task-id="1"] .island-row-text')?.click();
    await vi.waitFor(() => expect(root.querySelector("#edit-title")).not.toBeNull());
    expect(root.querySelector<HTMLSelectElement>("#edit-client")?.value).toBe("Altamira");

    const title = root.querySelector<HTMLInputElement>("#edit-title");
    const status = root.querySelector<HTMLSelectElement>("#edit-status");
    if (title && status) {
      title.value = "Tracking de formularios";
      status.value = "waiting_on_others";
    }
    await controller.refresh();
    expect(root.querySelector<HTMLInputElement>("#edit-title")?.value).toBe("Tracking de formularios");

    root.querySelector<HTMLFormElement>(".edit-form")?.requestSubmit();
    await vi.waitFor(() => expect(backend.editedTasks).toHaveLength(1));
    expect(backend.editedTasks[0]).toEqual({
      taskId: 1,
      edit: {
        title: "Tracking de formularios",
        notes: "",
        clientName: "Altamira",
        dueDate: "2026-10-22",
        priority: "normal",
        status: "waiting_on_others",
      },
    });
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("tasks"));
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
    await controller.changePreferences({ ...backend.savedPreferences, locale: "english" });

    expect(controller.currentState.mode).toBe("expanded");
    expect(controller.currentState.panel).toBe("settings");
    expect(root.querySelector(".island-head-title strong")?.textContent).toBe("Settings");
    expect(document.documentElement.lang).toBe("en");
  });

  it("changes reminder settings: focus mode, brief time and start with Windows", async () => {
    controller.openPanel("settings");
    root.querySelector<HTMLButtonElement>('[data-setting="settings.focus_mode"]')?.click();
    await vi.waitFor(() => expect(backend.savedPreferences.focusMode).toBe(true));

    const morning = root.querySelector<HTMLInputElement>("#setting-morningBriefTime");
    expect(morning?.value).toBe("08:30");
    if (morning) {
      morning.value = "07:45";
      morning.dispatchEvent(new Event("change"));
    }
    await vi.waitFor(() => expect(backend.savedPreferences.morningBriefTime).toBe("07:45"));

    root.querySelector<HTMLButtonElement>('[data-setting="settings.autostart"]')?.click();
    await vi.waitFor(() => expect(backend.autostart).toBe(true));
    await vi.waitFor(() =>
      expect(root.querySelector('[data-setting="settings.autostart"]')?.getAttribute("aria-checked")).toBe("true"));
  });

  it("shows the version, finds a new one and installs it from Settings", async () => {
    controller.openPanel("settings");
    await vi.waitFor(() => expect(root.querySelector(".settings-updates")?.textContent).toContain("Versión 0.2.0"));

    backend.availableUpdate = { version: "0.2.1", notes: null };
    root.querySelector<HTMLButtonElement>("#update-action")?.click();
    await vi.waitFor(() => expect(root.querySelector("#update-action")?.textContent).toBe("Instalar 0.2.1"));

    root.querySelector<HTMLButtonElement>("#update-action")?.click();
    await vi.waitFor(() => expect(backend.installedUpdates).toBe(1));
  });

  it("peeks out with the summary when a brief notification arrives", async () => {
    vi.useFakeTimers();
    backend.emit(SHELL_EVENTS.briefReady);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("compact"));

    vi.advanceTimersByTime(UI_CONFIG.briefPeekDurationMs);
    expect(notch()?.dataset.mode).toBe("hidden");
  });

  it("asks for a key before answering questions", () => {
    controller.openPanel("ask");
    expect(root.querySelector("#ask-input")).toBeNull();
    expect(root.querySelector(".ask .island-empty")?.textContent).toContain("Ajustes");
  });

  it("switches tabs instantly, without fading the card", () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-tab="agenda"]')?.click();
    expect(root.querySelector(".island-content-leaving")).toBeNull();
  });

  it("shows only Tiki while resting when the day is clear", async () => {
    controller.stop();
    const clearDay = { ...FakeBackend.sampleSnapshot(), attentionCount: 0, sections: [] };
    controller = await new TikiApp(root, new FakeBackend(clearDay)).start();

    expect(root.querySelector(".notch-hidden-busy")).toBeNull();
    expect(root.querySelector(".notch-hidden .notch-mascot")).not.toBeNull();
  });
});

describe("IslandController with the AI assistant", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let controller: IslandController;

  const claudeReads = (task: object) => () => ({
    id: "msg_test",
    type: "message",
    role: "assistant",
    model: "claude-opus-5-5",
    content: [{ type: "text", text: JSON.stringify(task) }],
    stop_reason: "end_turn",
    stop_sequence: null,
    usage: { input_tokens: 10, output_tokens: 10 },
  });

  beforeEach(async () => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend();
    backend.aiKeys.set("anthropic", "sk-test");
    controller = await new TikiApp(root, backend).start();
  });

  afterEach(() => controller.stop());

  it("shows what Tiki understood and saves it on the second Enter", async () => {
    backend.aiShell = new FakeShell(claudeReads({
      title: "Revisar acceso de facturas",
      clientName: "andeanvet",
      dueDate: "2026-10-23",
      priority: "high",
    }));
    controller.openPanel("capture");

    await controller.submitCapture("revisar acceso facturas andeanvet viernes, importante");
    const chips = [...root.querySelectorAll(".capture-chip")].map((chip) => chip.textContent);
    expect(chips).toContain("Cliente Andeanvet");
    expect(chips).toContain("Prioridad Alta");
    expect(backend.confirmedCaptures).toHaveLength(0);

    await controller.submitCapture("revisar acceso facturas andeanvet viernes, importante");
    expect(backend.confirmedCaptures).toEqual([
      { title: "Revisar acceso de facturas", clientName: "Andeanvet", dueDate: "2026-10-23", priority: "high" },
    ]);
    expect(backend.capturedSentences).toHaveLength(0);
    expect(controller.currentState.panel).toBe("tasks");
  });

  it("never sends the key to the interface's requests", async () => {
    backend.aiShell = new FakeShell(claudeReads({ title: "Llamar", clientName: null, dueDate: null, priority: "normal" }));
    await controller.submitCapture("llamar");

    const sent = JSON.stringify(backend.aiShell.requests);
    expect(sent).not.toContain("sk-test");
  });

  it("falls back to the core's rules when the assistant fails", async () => {
    backend.aiShell = new FakeShell(() => ({ type: "error", error: { type: "invalid_request_error", message: "bad" } }), 400);

    await controller.submitCapture("llamar a altamira");
    expect(backend.capturedSentences).toEqual(["llamar a altamira"]);
  });

  it("creates clients with the rules, without asking the assistant", async () => {
    const requestsBefore = backend.aiShell.requests.length;
    await controller.submitCapture("+Nuevo Cliente");
    expect(backend.capturedSentences).toEqual(["+Nuevo Cliente"]);
    expect(backend.aiShell.requests).toHaveLength(requestsBefore);
  });

  it("answers a question in the Ask panel with the open tasks as context", async () => {
    backend.aiShell = new FakeShell(claudeReads({ text: "Con Altamira tienes el tracking de formularios para mañana." }));
    controller.openPanel("ask");

    await vi.waitFor(() => expect(root.querySelector("#ask-input")).not.toBeNull());
    const input = root.querySelector<HTMLInputElement>("#ask-input");
    if (input) {
      input.value = "¿qué tengo con Altamira?";
      input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    }
    await vi.waitFor(() =>
      expect(root.querySelector(".ask-answer")?.textContent).toBe("Con Altamira tienes el tracking de formularios para mañana."));

    const body = backend.aiShell.lastBody();
    const sent = JSON.stringify(body.messages);
    expect(sent).toContain("Tracking de formularios");
    expect(sent).toContain("¿qué tengo con Altamira?");
  });

  it("keeps the day's AI brief on the Today tab", async () => {
    backend.aiShell = new FakeShell(claudeReads({ text: "Empieza por el tracking de Altamira." }));
    controller.openPanel("ask");
    await vi.waitFor(() => expect(root.querySelector("#ask-summarize")).not.toBeNull());
    root.querySelector<HTMLButtonElement>("#ask-summarize")?.click();
    await vi.waitFor(() => expect(controller.currentState.dayBrief).toBe("Empieza por el tracking de Altamira."));

    controller.selectTab("today");
    expect(root.querySelector(".island-brief")?.textContent).toBe("Empieza por el tracking de Altamira.");
  });

  it("shows the provider, the stored key and the model list in Settings", async () => {
    controller.openPanel("settings");
    await vi.waitFor(() => expect(root.querySelector(".settings-connected")).not.toBeNull());

    const models = [...root.querySelectorAll<HTMLOptionElement>("#ai-model-select option")].map((option) => option.value);
    expect(models).toContain("claude-opus-5-5");

    root.querySelector<HTMLButtonElement>(".settings-ai .settings-button-quiet")?.click();
    await vi.waitFor(() => expect(root.querySelector("#ai-key-input")).not.toBeNull());
    expect(backend.aiKeys.has("anthropic")).toBe(false);
  });

  it("saves a pasted key through the shell", async () => {
    backend.aiKeys.clear();
    controller.openPanel("settings");
    await vi.waitFor(() => expect(root.querySelector("#ai-key-input")).not.toBeNull());

    const input = root.querySelector<HTMLInputElement>("#ai-key-input");
    if (input) {
      input.value = "sk-new";
    }
    root.querySelector<HTMLButtonElement>(".settings-ai .settings-button")?.click();
    await vi.waitFor(() => expect(backend.aiKeys.get("anthropic")).toBe("sk-new"));
    await vi.waitFor(() => expect(root.querySelector(".settings-connected")).not.toBeNull());
  });
});
