import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SHELL_EVENTS, UI_CONFIG } from "../src/config/uiConfig";
import type { IslandController } from "../src/island/IslandController";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";
import { FakeShell } from "./support/FakeShell";

/** Types a sentence in quick capture, as the user would. */
function typeCapture(root: HTMLElement, sentence: string): void {
  const input = root.querySelector<HTMLInputElement>("#capture-input");
  if (input) {
    input.value = sentence;
  }
}

const EVENING_BRIEF = {
  kind: "evening" as const,
  title: "Cierre del día",
  body: "Atrasado: 1 · Vence pronto: 2 · Esperando: 2\nMañana vence: Tracking",
  taskId: null,
};

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

  it("expands on click and shows only today, with what to do now", () => {
    notch()?.click();

    const groups = [...root.querySelectorAll(".island-group")].map((group) => group.textContent);
    expect(groups).toEqual(["Atrasado", "Vence pronto", "Toca insistir"]);
    expect(root.querySelectorAll(".island-tab-content .island-row")).toHaveLength(4);
    expect(root.querySelector("#today-focus")?.textContent).toContain("Ahora · Está atrasada");
    expect(root.querySelector('[data-task-id="1"] .island-priority-high')?.textContent).toBe("!");
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
    expect(root.querySelector("#edit-project")?.textContent).toBe("Altamira");

    const title = root.querySelector<HTMLInputElement>("#edit-title");
    const status = root.querySelector<HTMLSelectElement>("#edit-status");
    const remind = root.querySelector<HTMLInputElement>("#edit-remind");
    if (title && status && remind) {
      title.value = "Tracking de formularios";
      status.value = "waiting";
      status.dispatchEvent(new Event("change"));
      remind.value = "2026-10-22T09:30";
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
        projectName: "Altamira",
        dueDate: "2026-10-22",
        priority: "normal",
        status: "waiting",
        remindAt: "2026-10-22T09:30:00",
        kind: "task",
        tags: [],
        repeat: "never",
        waitingFor: "",
      },
    });
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("tasks"));
  });

  it("lists every pending task by project in the Tasks tab, with search and project filter", () => {
    controller.selectTab("all_tasks");
    const groups = () => [...root.querySelectorAll(".tasks-group span:first-of-type")].map((group) => group.textContent);
    expect(groups()).toEqual(["Andeanvet", "Altamira", "Vapestation"]);

    const search = root.querySelector<HTMLInputElement>("#tasks-search");
    if (search) {
      search.value = "github";
      search.dispatchEvent(new Event("input"));
    }
    expect(root.querySelectorAll(".tasks-tab .island-row")).toHaveLength(1);

    if (search) {
      search.value = "";
      search.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>("#tasks-project-picker")?.click();
    const pickerSearch = root.querySelector<HTMLInputElement>(".picker-search");
    if (pickerSearch) {
      pickerSearch.value = "alta";
      pickerSearch.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>('.picker-option[data-value="Altamira"]')?.click();
    expect(groups()).toEqual(["Altamira"]);
    expect(root.querySelectorAll(".tasks-tab .island-row")).toHaveLength(2);
  });

  it("lists reminders apart from tasks, by time with their repeat", () => {
    controller.selectTab("all_tasks");
    expect(root.querySelector('.tasks-tab [data-task-id="8"]')).toBeNull();

    root.querySelector<HTMLButtonElement>('[data-view="reminders"]')?.click();
    expect(controller.currentState.tasksView).toBe("reminders");
    const rows = [...root.querySelectorAll(".tasks-tab .island-row")];
    expect(rows).toHaveLength(1);
    expect(rows[0]?.querySelector(".island-when")?.textContent).toBe("Vie 23 · 11:00 · Cada día");
  });

  it("keeps the search text when the island refreshes", async () => {
    controller.selectTab("all_tasks");
    const search = root.querySelector<HTMLInputElement>("#tasks-search");
    if (search) {
      search.value = "acta";
    }
    await controller.refresh();
    expect(root.querySelector<HTMLInputElement>("#tasks-search")?.value).toBe("acta");
  });

  it("shows the delivered history from Today and reopens a task from it", async () => {
    controller.setMode("expanded");
    await controller.refresh();
    root.querySelector<HTMLButtonElement>("#delivered-today")?.click();

    expect(controller.currentState.tab).toBe("all_tasks");
    expect(controller.currentState.tasksView).toBe("delivered");
    const rows = [...root.querySelectorAll(".island-row-delivered")].map((row) => row.querySelector(".island-when")?.textContent);
    expect(rows).toEqual(["hoy", "ayer"]);

    root.querySelector<HTMLButtonElement>('.island-row-delivered[data-task-id="9"] .island-row-text')?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("edit"));
  });

  it("creates, renames, recolors and removes projects in the project panel", async () => {
    controller.selectTab("all_tasks");
    await controller.refresh();
    root.querySelector<HTMLButtonElement>("#tasks-project-picker")?.click();
    [...root.querySelectorAll<HTMLButtonElement>(".picker-action")].find((action) => action.textContent === "Gestionar proyectos")?.click();
    expect(controller.currentState.panel).toBe("projects");

    const name = root.querySelector<HTMLInputElement>("#new-project");
    if (name) {
      name.value = "Constructora Sur";
    }
    root.querySelector<HTMLButtonElement>("#add-project")?.click();
    await vi.waitFor(() => expect(backend.projects.map((project) => project.name)).toContain("Constructora Sur"));
    await vi.waitFor(() => expect(root.querySelectorAll(".project-row")).toHaveLength(4));

    root.querySelector<HTMLButtonElement>('[data-project-id="3"] .island-icon-button')?.click();
    const rename = root.querySelector<HTMLInputElement>("#project-name-input");
    if (rename) {
      rename.value = "Vape Station";
      rename.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>('.project-swatch[data-color="#7357E8"]')?.click();
    root.querySelector<HTMLButtonElement>("#save-project")?.click();
    await vi.waitFor(() =>
      expect(backend.projects.find((project) => project.projectId === 3)).toMatchObject({ name: "Vape Station", color: "#7357E8" }));

    root.querySelector<HTMLButtonElement>('[data-project-id="1"] .project-remove')?.click();
    root.querySelector<HTMLButtonElement>('[data-project-id="1"] .project-remove-confirm')?.click();
    await vi.waitFor(() => expect(backend.projects.some((project) => project.projectId === 1)).toBe(false));
  });

  it("saves a repeating reminder from the box, with only its moment and repeat", async () => {
    controller.openPanel("capture");
    typeCapture(root, "Tomar la pastilla");
    root.querySelector<HTMLButtonElement>('[data-kind="reminder"]')?.click();
    expect(root.querySelector("#capture-due")).toBeNull();
    const remind = root.querySelector<HTMLInputElement>("#capture-remind");
    const repeat = root.querySelector<HTMLSelectElement>("#capture-repeat");
    if (remind && repeat) {
      remind.value = "2026-10-22T08:00";
      repeat.value = "daily";
    }
    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ kind: "reminder", remindAt: "2026-10-22T08:00:00", repeat: "daily", tags: [] });
  });

  it("creates a new project from the picker by typing its name", async () => {
    controller.openPanel("capture");
    typeCapture(root, "Preparar propuesta");
    root.querySelector<HTMLButtonElement>("#capture-project")?.click();
    const search = root.querySelector<HTMLInputElement>(".picker-search");
    if (search) {
      search.value = "Constructora Sur";
      search.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>(".picker-create")?.click();
    expect(root.querySelector("#capture-project")?.textContent).toBe("Constructora Sur");

    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ projectName: "Constructora Sur" });
  });

  it("keeps notes for a project as context for Tiki", async () => {
    await controller.refresh();
    controller.openPanel("projects");
    root.querySelector<HTMLButtonElement>('[data-project-id="1"] .island-icon-button')?.click();
    const notes = root.querySelector<HTMLTextAreaElement>("#project-notes-input");
    if (notes) {
      notes.value = "Contacto: Lucía. Prefieren correos cortos.";
      notes.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>("#save-project")?.click();
    await vi.waitFor(() => expect(backend.projects.find((project) => project.projectId === 1)?.notes).toBe("Contacto: Lucía. Prefieren correos cortos."));
  });

  it("does not fold while the user is typing a search", () => {
    vi.useFakeTimers();
    controller.selectTab("all_tasks");
    root.querySelector<HTMLInputElement>("#tasks-search")?.focus();
    notch()?.dispatchEvent(new Event("pointerleave"));

    vi.advanceTimersByTime(UI_CONFIG.foldDelayMs);
    expect(notch()?.dataset.mode).toBe("expanded");
  });

  it("shows Today, Tasks and Agenda, and no Tickets tab until a ticket connector exists", () => {
    controller.setMode("expanded");
    const tabs = [...root.querySelectorAll<HTMLElement>(".island-tab[data-tab]")].map((tab) => tab.dataset.tab);
    expect(tabs).toEqual(["today", "all_tasks", "agenda"]);

    root.querySelector<HTMLButtonElement>('[data-tab="agenda"]')?.click();
    expect(controller.currentState.tab).toBe("agenda");
    expect(root.querySelector(".island-empty")).not.toBeNull();
  });

  it("opens quick capture when the shell asks for it", () => {
    backend.emit(SHELL_EVENTS.openCapture);
    expect(root.querySelector("#capture-input")).not.toBeNull();
  });

  it("saves the sentence as typed with the fields the user picked when there is no AI", async () => {
    controller.openPanel("capture");
    typeCapture(root, "revisar acta altamira viernes");
    root.querySelector<HTMLButtonElement>("#capture-project")?.click();
    root.querySelector<HTMLButtonElement>('.picker-option[data-value="Altamira"]')?.click();
    const due = root.querySelector<HTMLInputElement>("#capture-due");
    if (due) {
      due.value = "2026-10-23";
    }
    expect(root.querySelector(".capture-status")?.textContent).toContain("Configurar IA");

    await controller.submitCapture();
    expect(backend.confirmedCaptures).toEqual([
      {
        title: "revisar acta altamira viernes",
        projectName: "Altamira",
        dueDate: "2026-10-23",
        priority: "normal",
        remindAt: null,
        kind: "task",
        tags: [],
        repeat: "never",
        waitingFor: null,
      },
    ]);
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

  it("changes reminder settings: focus mode and brief time", async () => {
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
  });

  it("shows Settings without the big mascot on the narrow left and right island", async () => {
    await controller.changePreferences({ ...backend.savedPreferences, islandPlacement: "left" });
    controller.openPanel("settings");
    expect(root.querySelector(".island-card")?.getAttribute("data-panel")).toBe("settings");
    expect(root.dataset.orientation).toBe("vertical");
  });

  it("keeps only the island's own settings and links to the advanced ones", () => {
    controller.openPanel("settings");
    const groups = [...root.querySelectorAll(".settings-group h3")].map((title) => title.textContent);
    expect(groups).toEqual(["Posición de la isla", "Idioma", "Recordatorios"]);
    expect(root.querySelector(".settings-ai")).toBeNull();

    root.querySelector<HTMLButtonElement>("#open-advanced-settings")?.click();
    expect(backend.openedSettings).toEqual([undefined]);
    expect(notch()?.dataset.mode).toBe("hidden");
  });

  it("reloads preferences changed in the advanced settings window", async () => {
    backend.savedPreferences = { ...backend.savedPreferences, locale: "english" };
    backend.emit(SHELL_EVENTS.preferencesChanged);
    await vi.waitFor(() => expect(document.documentElement.lang).toBe("en"));
  });

  it("shows a reminder in the grown notch, then folds leaving an unread dot", async () => {
    vi.useFakeTimers();
    backend.deliverAlert(EVENING_BRIEF);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));
    expect(root.querySelector(".notch-alert strong")?.textContent).toBe("Cierre del día");
    expect(root.querySelector(".notch-alert-preview")?.textContent).toBe(
      "Atrasado: 1 · Vence pronto: 2 · Esperando: 2 · Mañana vence: Tracking",
    );

    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(notch()?.dataset.mode).toBe("hidden");
    expect(root.querySelector(".notch-unread")).not.toBeNull();
  });

  it("opens the brief as a receipt on click and puts it away after reading", async () => {
    backend.deliverAlert(EVENING_BRIEF);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));

    notch()?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("brief"));
    expect(root.querySelector(".receipt-head strong")?.textContent).toBe("Cierre del día");
    expect(root.querySelectorAll(".receipt-block")).toHaveLength(3);
    expect(root.querySelector(".receipt-tip")?.textContent).toContain("Mañana vence: Tracking");
    expect(root.querySelector(".receipt-foot")?.textContent).toBe("6 pendientes · 2 esperando · 1 entregadas hoy");
    expect(controller.currentState.unreadAlert).toBe(false);

    controller.setMode("hidden");
    expect(controller.currentState.alert).toBeNull();
    expect(root.querySelector(".notch-unread")).toBeNull();
  });

  it("opens the task of a task reminder", async () => {
    backend.deliverAlert({ kind: "task", title: "Recordatorio", body: "Tracking de formularios\nAltamira · mañana", taskId: 1 });
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));

    notch()?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("edit"));
    expect(root.querySelector<HTMLInputElement>("#edit-title")?.value).toBe("Tracking de formularios comerciales");
  });

  it("only adds the reminder to Today when the island is already open", async () => {
    controller.setMode("expanded");
    backend.deliverAlert(EVENING_BRIEF);
    await vi.waitFor(() => expect(root.querySelector(".island-notice")).not.toBeNull());
    expect(notch()?.dataset.mode).toBe("expanded");
  });

  it("lets the user mark a reminder done or snooze it from Today", async () => {
    controller.setMode("expanded");
    backend.deliverAlert({ kind: "reminder", title: "Recordatorio", body: "Llamar a mamá", taskId: 7 });
    await vi.waitFor(() => expect(root.querySelector("#alert-snooze-10")).not.toBeNull());

    root.querySelector<HTMLButtonElement>("#alert-snooze-10")?.click();
    await vi.waitFor(() => expect(backend.snoozed).toEqual([{ taskId: 7, minutes: 10 }]));
    await vi.waitFor(() => expect(root.querySelector(".island-notice")).toBeNull());
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

  const captureReply = (item: object) => claudeReads({ intent: "capture", item, answer: null, actions: [] });
  const answerReply = (answer: string, actions: object[] = []) => claudeReads({ intent: "answer", item: null, answer, actions });
  const item = (fields: object) => ({
    kind: "task",
    title: "",
    projectName: null,
    dueDate: null,
    remindAt: null,
    repeat: "never",
    waitingFor: null,
    tags: [],
    priority: "normal",
    ...fields,
  });
  const write = async (text: string) => {
    controller.openPanel("capture");
    typeCapture(root, text);
    await controller.submitCapture();
  };

  it("fills the fields of an item with AI and saves them on the second Enter, learning from changes", async () => {
    backend.aiShell = new FakeShell(captureReply(item({
      title: "Revisar acceso de facturas",
      projectName: "andeanvet",
      dueDate: "2026-10-23",
      priority: "high",
      remindAt: "2026-10-23T09:00",
      tags: ["accesos"],
    })));
    await write("revisar acceso facturas andeanvet viernes, importante, avísame a las 9");
    expect(root.querySelector<HTMLInputElement>("#capture-input")?.value).toBe("Revisar acceso de facturas");
    expect(root.querySelector("#capture-project")?.textContent).toBe("Andeanvet");
    expect(root.querySelector<HTMLInputElement>("#capture-due")?.value).toBe("2026-10-23");
    expect(root.querySelector<HTMLSelectElement>("#capture-priority")?.value).toBe("high");
    expect(backend.confirmedCaptures).toHaveLength(0);

    const due = root.querySelector<HTMLInputElement>("#capture-due");
    if (due) {
      due.value = "2026-10-24";
    }
    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ title: "Revisar acceso de facturas", projectName: "Andeanvet", dueDate: "2026-10-24" });
    await vi.waitFor(() => expect(backend.lessons).toHaveLength(1));
    expect(backend.lessons[0]).toMatchObject({ understood: { dueDate: "2026-10-23" }, corrected: { dueDate: "2026-10-24" } });
    expect(controller.currentState.panel).toBe("tasks");
  });

  it("reads a repeating reminder and shows only its moment and repeat", async () => {
    backend.aiShell = new FakeShell(captureReply(item({
      kind: "reminder",
      title: "Tomar la pastilla",
      remindAt: "2026-10-22T08:00",
      repeat: "daily",
    })));
    await write("recuérdame tomar la pastilla cada día a las 8");
    expect(root.querySelector("#capture-due")).toBeNull();
    expect(root.querySelector<HTMLSelectElement>("#capture-repeat")?.value).toBe("daily");

    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ kind: "reminder", repeat: "daily", remindAt: "2026-10-22T08:00:00", dueDate: null });
  });

  it("keeps who a task waits for as free text and a new project by name", async () => {
    backend.aiShell = new FakeShell(captureReply(item({
      title: "Contrato firmado",
      projectName: "Constructora Sur",
      waitingFor: "respuesta de Lucía",
    })));
    await write("esperando respuesta de Lucía sobre el contrato de Constructora Sur");
    expect(root.querySelector("#capture-project")?.textContent).toBe("Constructora Sur");
    expect(root.querySelector<HTMLInputElement>("#capture-waiting")?.value).toBe("respuesta de Lucía");

    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ projectName: "Constructora Sur", waitingFor: "respuesta de Lucía" });
  });

  it("answers a question in the same box, with the open items as context", async () => {
    backend.aiShell = new FakeShell(answerReply("Con Altamira tienes el tracking de formularios para mañana."));
    await write("¿qué tengo con Altamira?");

    expect(root.querySelector(".capture-answer .ask-answer")?.textContent).toBe("Con Altamira tienes el tracking de formularios para mañana.");
    expect(root.querySelector<HTMLElement>(".capture-fields")?.hidden).toBe(true);
    expect(backend.confirmedCaptures).toHaveLength(0);
    const sent = JSON.stringify(backend.aiShell.lastBody().messages);
    expect(sent).toContain("Tracking de formularios");
    expect(sent).toContain("¿qué tengo con Altamira?");
  });

  it("applies the actions Tiki proposes only when the user clicks", async () => {
    backend.aiShell = new FakeShell(answerReply("Listo, te propongo esto.", [
      { type: "set_due_date", summary: "Tracking vence el viernes", taskId: 1, title: null, projectName: null, dueDate: "2026-10-23", remindAt: null, status: null },
      { type: "create_reminder", summary: "Avisarte mañana a las 10", taskId: null, title: "Llamar a Lucía", projectName: "Altamira", dueDate: null, remindAt: "2026-10-22T10:00", status: null },
    ]));
    await write("mueve el tracking al viernes y recuérdame llamar a Lucía mañana a las 10");
    expect(root.querySelectorAll(".ask-action")).toHaveLength(2);
    expect(backend.editedTasks).toHaveLength(0);
    expect(backend.confirmedCaptures).toHaveLength(0);

    root.querySelector<HTMLButtonElement>(".ask-apply-all")?.click();
    await vi.waitFor(() => expect(backend.editedTasks).toHaveLength(1));
    expect(backend.editedTasks[0]?.edit.dueDate).toBe("2026-10-23");
    await vi.waitFor(() => expect(backend.confirmedCaptures).toHaveLength(1));
    expect(backend.confirmedCaptures[0]).toMatchObject({ kind: "reminder", title: "Llamar a Lucía", remindAt: "2026-10-22T10:00:00" });
    await vi.waitFor(() => expect(root.querySelectorAll(".ask-action-done")).toHaveLength(2));
  });

  it("never sends the key to the interface's requests", async () => {
    backend.aiShell = new FakeShell(captureReply(item({ title: "Llamar" })));
    await write("llamar");
    expect(JSON.stringify(backend.aiShell.requests)).not.toContain("sk-test");
  });

  it("lets the user fill the fields when the assistant fails, without guessing", async () => {
    backend.aiShell = new FakeShell(() => ({ type: "error", error: { type: "invalid_request_error", message: "bad" } }), 400);
    await write("llamar a altamira");
    expect(root.querySelector(".capture-status")?.textContent).toContain("No pude");
    expect(backend.confirmedCaptures).toHaveLength(0);

    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ title: "llamar a altamira", projectName: null, kind: "task" });
  });

  it("writes the day's AI brief in the summary card of Today", async () => {
    backend.aiShell = new FakeShell(claudeReads({ text: "Empieza por el tracking de Altamira." }));
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>("#prepare-brief")?.click();
    await vi.waitFor(() => expect(controller.currentState.dayBrief).toBe("Empieza por el tracking de Altamira."));
    expect(root.querySelector("#today-summary")?.textContent).toBe("Empieza por el tracking de Altamira.");
  });

});
