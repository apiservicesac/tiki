import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { BackendError } from "../src/backend/BackendError";
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
  summary: "Atrasado: 1 · Vence pronto: 2 · Esperando: 2",
  tips: ["Mañana vence: Tracking"],
  actions: ["open" as const],
  taskId: null,
  link: null,
  origin: null,
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

  it("shows Tiki's face and the day's summary while resting when work is pending", () => {
    expect(root.querySelector(".notch-hidden-busy .mascot-worried")).not.toBeNull();
    expect(root.querySelector(".notch-hidden-busy .notch-summary")?.textContent).toBe("3 por atender · 2 esperando");
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
    expect(root.querySelector(".notch-summary")?.textContent).toBe("3 por atender · 2 esperando");

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
      input.value = "llamar a acme";
    }
    await controller.refresh();
    expect(root.querySelector<HTMLInputElement>("#capture-input")?.value).toBe("llamar a acme");
  });

  it("expands on click and shows only today, with what to do now", () => {
    notch()?.click();

    const groups = [...root.querySelectorAll(".island-group")].map((group) => group.textContent);
    // The overdue task is "Now", so it is not listed again below it.
    expect(groups).toEqual(["Vence pronto", "Toca insistir"]);
    expect(root.querySelectorAll(".island-tab-content .island-row")).toHaveLength(3);
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
    expect(root.querySelector("#edit-project")?.textContent).toBe("Acme");

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
        projectName: "Acme",
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
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("tabs"));
  });

  it("fades a new tab in, but redraws the same tab instantly", async () => {
    controller.setMode("expanded");
    expect(root.querySelector(".island-tab-entering")).toBeNull();
    controller.selectTab("agenda");
    expect(root.querySelector(".island-card-body.island-tab-entering")).not.toBeNull();
    await controller.refresh();
    expect(root.querySelector(".island-tab-entering")).toBeNull();
  });

  it("lists every pending task by project in the Tasks tab, with search and project filter", () => {
    controller.selectTab("all_tasks");
    const groups = () => [...root.querySelectorAll(".tasks-group span:first-of-type")].map((group) => group.textContent);
    expect(groups()).toEqual(["Atlas", "Acme", "Vertex"]);

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
      pickerSearch.value = "acm";
      pickerSearch.dispatchEvent(new Event("input"));
    }
    root.querySelector<HTMLButtonElement>('.picker-option[data-value="1"]')?.click();
    expect(groups()).toEqual(["Acme"]);
    expect(root.querySelectorAll(".tasks-tab .island-row")).toHaveLength(2);
  });

  it("shows reminders in the Agenda by day, and only tasks in Tasks", () => {
    controller.selectTab("all_tasks");
    expect(root.querySelector('.tasks-tab [data-task-id="8"]')).toBeNull();
    const views = [...root.querySelectorAll<HTMLElement>("[data-view]")].map((view) => view.dataset.view);
    expect(views).toEqual(["pending", "delivered", "trash"]);

    controller.selectTab("agenda");
    const row = root.querySelector('.agenda-row[data-task-id="8"]');
    expect(row?.querySelector(".agenda-time")?.textContent).toBe("11:00");
    expect(row?.querySelector("small")?.textContent).toBe("Cada día");
    expect(root.querySelector(".agenda-list .island-group")?.textContent).not.toBe("");
  });

  it("puts calendar meetings in the Agenda, by time, with a Join button", async () => {
    controller.selectTab("agenda");
    const rows = [...root.querySelectorAll<HTMLElement>(".agenda-row")];
    expect(rows.map((row) => row.querySelector(".agenda-time")?.textContent)).toEqual(["09:00–09:30", "11:00", "16:00"]);
    expect(rows[0]?.querySelector(".island-row-text")?.textContent).toBe("Daily de AcmeTeams");

    rows[0]?.querySelector<HTMLButtonElement>(".agenda-join")?.click();
    await vi.waitFor(() => expect(backend.openedLinks).toEqual(["https://teams.microsoft.com/l/meetup-join/daily"]));
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

    const name = root.querySelector<HTMLInputElement>("#projects-search");
    if (name) {
      name.value = "Constructora Sur";
      name.dispatchEvent(new Event("input"));
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

  it("says what to fix when the core refuses a capture, keeping what the user wrote", async () => {
    backend.saveFailure = new BackendError("reminder_needs_time", "a reminder needs a time to ring");
    controller.openPanel("capture");
    typeCapture(root, "Llamar a mamá");
    root.querySelector<HTMLButtonElement>('[data-kind="reminder"]')?.click();
    await controller.submitCapture();

    expect(root.querySelector(".capture-status-error")?.textContent).toBe("Un recordatorio necesita fecha y hora para sonar.");
    expect(root.querySelector<HTMLInputElement>("#capture-input")?.value).toBe("Llamar a mamá");
    expect(controller.currentState.panel).toBe("capture");
  });

  it("says what went wrong when the core refuses an edit, keeping the form filled", async () => {
    backend.saveFailure = new BackendError("database", "database error: disk full");
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-task-id="1"] .island-row-text')?.click();
    await vi.waitFor(() => expect(root.querySelector("#edit-title")).not.toBeNull());
    const title = root.querySelector<HTMLInputElement>("#edit-title");
    if (title) {
      title.value = "Tracking de formularios";
    }
    root.querySelector<HTMLFormElement>(".edit-form")?.requestSubmit();

    await vi.waitFor(() => expect(root.querySelector<HTMLElement>(".edit-problem")?.hidden).toBe(false));
    expect(root.querySelector(".edit-problem")?.textContent).toBe("No se pudo guardar. Inténtalo de nuevo.");
    expect(root.querySelector<HTMLInputElement>("#edit-title")?.value).toBe("Tracking de formularios");
    expect(controller.currentState.panel).toBe("edit");
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

  it("shows Today, Tasks, Agenda and Sessions, and no Tickets tab until a ticket connector exists", () => {
    controller.setMode("expanded");
    const tabs = [...root.querySelectorAll<HTMLElement>(".island-tab[data-tab]")].map((tab) => tab.dataset.tab);
    expect(tabs).toEqual(["today", "all_tasks", "agenda", "sessions"]);

    root.querySelector<HTMLButtonElement>('[data-tab="agenda"]')?.click();
    expect(controller.currentState.tab).toBe("agenda");
    expect(root.querySelector(".agenda-list")).not.toBeNull();
  });

  it("opens quick capture when the shell asks for it", () => {
    backend.emit(SHELL_EVENTS.openCapture);
    expect(root.querySelector("#capture-input")).not.toBeNull();
  });

  it("saves the sentence as typed with the fields the user picked when there is no AI", async () => {
    controller.openPanel("capture");
    typeCapture(root, "revisar acta acme viernes");
    root.querySelector<HTMLButtonElement>("#capture-project")?.click();
    root.querySelector<HTMLButtonElement>('.picker-option[data-value="Acme"]')?.click();
    const due = root.querySelector<HTMLInputElement>("#capture-due");
    if (due) {
      due.value = "2026-10-23";
    }
    expect(root.querySelector(".capture-status")?.textContent).toContain("Configurar IA");

    await controller.submitCapture();
    expect(backend.confirmedCaptures).toEqual([
      {
        title: "revisar acta acme viernes",
        projectName: "Acme",
        dueDate: "2026-10-23",
        priority: "normal",
        remindAt: null,
        kind: "task",
        tags: [],
        repeat: "never",
        waitingFor: null,
      },
    ]);
    expect(controller.currentState.panel).toBe("tabs");
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
    await vi.waitFor(() => expect(backend.savedPreferences.morningBriefTime).toBe("07:45:00"));
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
    expect(groups).toEqual(["Dónde aparece Tiki", "Idioma", "Notch", "Avisos", "Silencio", "Resúmenes"]);
    expect(root.querySelector(".settings-ai")).toBeNull();

    root.querySelector<HTMLButtonElement>("#open-advanced-settings")?.click();
    expect(backend.openedSettings).toEqual([undefined]);
    expect(notch()?.dataset.mode).toBe("hidden");
  });

  it("explains every alert option and dims the ones that depend on alerts while they are off", async () => {
    controller.openPanel("settings");
    const rows = [...root.querySelectorAll(".setting-row")];
    expect(rows.every((row) => row.querySelector(".setting-row-text small")?.textContent)).toBe(true);

    await controller.changePreferences({ ...backend.savedPreferences, notificationsEnabled: false });
    const sound = root.querySelector('[data-setting="settings.alert_sound"]')?.closest(".setting-row");
    expect(sound?.classList.contains("setting-row-inactive")).toBe(true);
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

  it("announces an alert with its kind, its own face and color, then goes back to the day's mood", async () => {
    vi.useFakeTimers();
    backend.deliverAlert({ kind: "meeting", title: "Daily", summary: "En 15 min", tips: [], actions: ["open"], taskId: null, link: null, origin: null });
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));
    expect(root.querySelector(".notch-alert")?.getAttribute("data-alert-kind")).toBe("meeting");
    expect(root.querySelector(".notch-alert-kind")?.textContent).toBe("Reunión");
    expect(notch()?.dataset.mood).toBe(UI_CONFIG.alertStyles.meeting.mood);
    expect(root.dataset.mood).toBe(UI_CONFIG.alertStyles.meeting.mood);
    expect(root.querySelector(".notch-alert-countdown")).not.toBeNull();

    vi.advanceTimersByTime(UI_CONFIG.alertDurationMs);
    expect(notch()?.dataset.mood).toBe("worried");
  });

  it("sends the task being edited to the trash and goes back to the tabs", async () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-task-id="1"] .island-row-text')?.click();
    await vi.waitFor(() => expect(root.querySelector("#edit-trash")).not.toBeNull());
    root.querySelector<HTMLButtonElement>("#edit-trash")?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("tabs"));
    expect(backend.trash.some((item) => item.taskId === 1)).toBe(true);
  });

  it("keeps the alert in the notch untouched while other things redraw the island", async () => {
    backend.deliverAlert(EVENING_BRIEF);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));
    const shown = root.querySelector(".notch-alert-countdown");
    window.dispatchEvent(new Event("resize"));
    await controller.refresh();
    expect(root.querySelector(".notch-alert-countdown")).toBe(shown);
  });

  it("opens the brief as a receipt on click and puts it away after reading", async () => {
    backend.deliverAlert(EVENING_BRIEF);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));

    notch()?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("brief"));
    expect(root.querySelector(".receipt-head strong")?.textContent).toBe("Cierre del día");
    expect(root.querySelectorAll(".receipt-block")).toHaveLength(3);
    expect(root.querySelector(".receipt-tip")?.textContent).toContain("Mañana vence: Tracking");
    expect(root.querySelector(".receipt-foot")?.textContent).toBe("Pendientes: 6 · Esperando: 2 · Entregadas hoy: 1");
    expect(controller.currentState.unreadAlert).toBe(false);

    controller.setMode("hidden");
    expect(controller.currentState.alert).toBeNull();
    expect(root.querySelector(".notch-unread")).toBeNull();
  });

  it("opens the task of a task reminder", async () => {
    backend.deliverAlert({ kind: "task", title: "Recordatorio", summary: "Tracking de formularios", tips: ["Acme · mañana"], actions: ["open", "snooze"], taskId: 1, link: null, origin: null });
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
    backend.deliverAlert({ kind: "reminder", title: "Recordatorio", summary: "Llamar a mamá", tips: [], actions: ["open", "done", "snooze"], taskId: 7, link: null, origin: null });
    await vi.waitFor(() => expect(root.querySelector("#alert-snooze-10")).not.toBeNull());

    root.querySelector<HTMLButtonElement>("#alert-snooze-10")?.click();
    await vi.waitFor(() => expect(backend.snoozed).toEqual([{ taskId: 7, minutes: 10 }]));
    await vi.waitFor(() => expect(root.querySelector(".island-notice")).toBeNull());
  });

  it("offers Join on a meeting alert and opens the call", async () => {
    controller.setMode("expanded");
    backend.deliverAlert({ kind: "meeting", title: "Reunión en 15 min", summary: "Daily", tips: ["09:00–09:30"], actions: ["open", "join"], taskId: null, link: "https://meet.google.com/abc", origin: null });
    await vi.waitFor(() => expect(root.querySelector("#alert-join")).not.toBeNull());

    root.querySelector<HTMLButtonElement>("#alert-join")?.click();
    await vi.waitFor(() => expect(backend.openedLinks).toEqual(["https://meet.google.com/abc"]));
    expect(controller.currentState.alert).toBeNull();
  });

  it("shows in the notch whose account a meeting alert comes from and its platform", async () => {
    const account = { provider: "google" as const, email: "ana@acme.com", displayName: "Ana Pérez", avatar: "data:image/jpeg;base64,/9j/4AAQ" };
    const origin = { account, platform: "google_meet" as const, label: "Google Meet · Ana Pérez · ana@acme.com" };
    backend.deliverAlert({ kind: "meeting", title: "Reunión en 15 min", summary: "Daily", tips: [], actions: ["open"], taskId: null, link: null, origin });
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));

    const shown = notch()?.querySelector(".notch-alert-origin");
    expect(shown?.getAttribute("title")).toBe(origin.label);
    expect(shown?.textContent).toBe("ana@acme.com");
    expect(shown?.querySelector("img")?.getAttribute("src")).toBe(account.avatar);
    expect(shown?.querySelector<HTMLElement>(".account-service")?.dataset.service).toBe("google_meet");
  });

  it("opens the Agenda from a meeting alert in the notch", async () => {
    backend.deliverAlert({ kind: "meeting", title: "Reunión en 15 min", summary: "Daily", tips: [], actions: ["open"], taskId: null, link: null, origin: null });
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));

    notch()?.click();
    await vi.waitFor(() => expect(controller.currentState.tab).toBe("agenda"));
  });

  it("switches tabs instantly, without fading the card", () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[data-tab="agenda"]')?.click();
    expect(root.querySelector(".island-content-leaving")).toBeNull();
  });

  it("records only after the user picks a mode, reminding them to tell the participants", async () => {
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[aria-label="Grabar"]')?.click();
    expect(controller.currentState.panel).toBe("record");
    expect(root.querySelector(".record-consent")?.textContent).toBe("Antes de grabar, avisa a los participantes.");
    expect(backend.recordingRequests).toEqual([]);

    root.querySelector<HTMLButtonElement>('.record-option[data-mode="audio_only"]')?.click();
    await vi.waitFor(() => expect(root.querySelector(".island-head .recording-indicator")).not.toBeNull());
    expect(backend.recordingRequests).toEqual(["start:audio_only"]);
    expect(controller.currentState.panel).toBe("tabs");
  });

  it("records a screen or a window in the quality chosen, and a video does not pause", async () => {
    controller.openPanel("record");
    root.querySelector<HTMLButtonElement>('.record-option[data-mode="audio_and_video"]')?.click();
    await vi.waitFor(() => expect(root.querySelector(".record-target")).not.toBeNull());
    const targets = [...root.querySelectorAll(".record-target")].map((target) => target.textContent);
    expect(targets).toEqual(["Pantalla 1", "Reunión de Teams"]);
    expect(backend.recordingRequests).toEqual([]);

    root.querySelector<HTMLButtonElement>('[data-quality="high"]')?.click();
    root.querySelector<HTMLButtonElement>('.record-target[data-kind="window"]')?.click();
    await vi.waitFor(() => expect(backend.recordingRequests).toEqual(["start:audio_and_video"]));
    expect(backend.lastVideoChoice).toEqual({ kind: "window", id: 1_311_754, quality: "high" });
    await vi.waitFor(() => expect(root.querySelector(".island-head .recording-indicator")).not.toBeNull());
    expect(root.querySelector('[aria-label="Pausar la grabación"]')).toBeNull();
  });

  it("keeps the red dot and the timer in the folded notch while recording", async () => {
    vi.useFakeTimers();
    await backend.startRecording("audio_only", null);
    await controller.refresh();
    expect(notch()?.dataset.mode).toBe("recording");
    expect(root.querySelector(".notch-recording .recording-timer")?.textContent).toBe("00:00");
    await vi.advanceTimersByTimeAsync(UI_CONFIG.recordingTimerTickMs * 65);
    expect(root.querySelector(".notch-recording .recording-timer")?.textContent).toBe("01:05");

    await backend.stopRecording();
    await controller.refresh();
    expect(notch()?.dataset.mode).toBe("hidden");
  });

  it("pauses, resumes and stops from the header", async () => {
    await backend.startRecording("audio_only", null);
    await controller.refresh();
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>('[aria-label="Pausar la grabación"]')?.click();
    await vi.waitFor(() => expect(root.querySelector('.recording-indicator[data-paused="true"]')).not.toBeNull());
    root.querySelector<HTMLButtonElement>('[aria-label="Seguir grabando"]')?.click();
    await vi.waitFor(() => expect(root.querySelector('.recording-indicator[data-paused="false"]')).not.toBeNull());
    root.querySelector<HTMLButtonElement>('[aria-label="Terminar la grabación"]')?.click();
    await vi.waitFor(() => expect(root.querySelector(".recording-indicator")).toBeNull());
    expect(backend.recordingRequests).toEqual(["start:audio_only", "pause", "resume", "stop"]);
  });

  it("stops with the shortcut while recording, and otherwise asks what to record", async () => {
    backend.emit(SHELL_EVENTS.toggleRecording);
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("record"));
    await backend.startRecording("audio_only", null);
    await controller.refresh();
    backend.emit(SHELL_EVENTS.toggleRecording);
    await vi.waitFor(() => expect(backend.recordingRequests).toEqual(["start:audio_only", "stop"]));
  });

  const recordingOffer = { kind: "recording" as const, title: "¿Grabar esta llamada?", summary: "Hay una llamada en curso.", tips: ["Si grabas, avisa a los participantes."], actions: ["open" as const, "record_audio" as const, "record_video" as const], taskId: null, link: null, origin: null };
  const quietly = { sound: false, windowsNotification: false };

  it("opens the choice of what to record from Tiki's offer when a call starts", async () => {
    backend.deliverAlert(recordingOffer, quietly);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));
    expect(root.querySelector(".notch-alert-kind")?.textContent).toBe("Grabar");

    notch()?.click();
    await vi.waitFor(() => expect(controller.currentState.panel).toBe("record"));
    expect(backend.recordingRequests).toEqual([]);
  });

  it("records audio from the offer's button in the notch and rests with the red dot", async () => {
    backend.deliverAlert(recordingOffer, quietly);
    await vi.waitFor(() => expect(root.querySelector("#notch-alert-record-audio")).not.toBeNull());
    expect(root.querySelector("#notch-alert-record-video")?.textContent).toBe("Audio y video");

    root.querySelector<HTMLButtonElement>("#notch-alert-record-audio")?.click();
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("recording"));
    expect(backend.recordingRequests).toEqual(["start:audio_only"]);
    expect(controller.currentState.alert).toBeNull();
  });

  it("asks which screen or window to record from the offer's video button", async () => {
    backend.deliverAlert(recordingOffer, quietly);
    await vi.waitFor(() => expect(root.querySelector("#notch-alert-record-video")).not.toBeNull());
    root.querySelector<HTMLButtonElement>("#notch-alert-record-video")?.click();
    await vi.waitFor(() => expect(root.querySelector(".record-target")).not.toBeNull());
    expect(backend.recordingRequests).toEqual([]);
  });

  it("lists the recorded sessions with their status and what their media takes", () => {
    controller.selectTab("sessions");
    const rows = [...root.querySelectorAll<HTMLElement>(".session-row")];
    expect(rows.map((row) => row.dataset.sessionId)).toEqual(["2", "1"]);
    expect(rows[0]?.querySelector(".island-when")?.textContent).toBe("Lista");
    expect(rows[0]?.querySelector("small")?.textContent).toBe("Mié 21 · 10:00 · 30:30");
    expect(rows[1]?.querySelector(".session-retry")).not.toBeNull();
    expect(root.querySelector(".sessions-storage")?.textContent).toBe("El audio y el video ocupan 24 MB y se borran a los 90 días.");
  });

  it("retries a transcription that failed", async () => {
    controller.selectTab("sessions");
    root.querySelector<HTMLButtonElement>(".session-retry")?.click();
    await vi.waitFor(() => expect(backend.retriedTranscriptions).toEqual([1]));
  });

  it("opens a session with its player and the transcript labelled You and Others", async () => {
    controller.selectTab("sessions");
    root.querySelector<HTMLButtonElement>('.session-row[data-session-id="2"] .island-row-text')?.click();
    await vi.waitFor(() => expect(root.querySelector(".session-detail")).not.toBeNull());

    const player = root.querySelector<HTMLVideoElement>("video.session-player");
    expect(player?.getAttribute("src")).toBe("http://asset.localhost/C%3A%5CTiki%5Csessions%5C2%5Csession.mp4");
    const speakers = [...root.querySelectorAll(".session-line-speaker")].map((speaker) => speaker.textContent);
    expect(speakers).toEqual(["Otros", "Tú", "Otros"]);
  });

  it("follows the player in the transcript and plays from a clicked line", async () => {
    await controller.openSession(2);
    const player = root.querySelector<HTMLVideoElement>("video.session-player");
    if (!player) {
      throw new Error("no player");
    }
    player.play = () => Promise.resolve();
    root.querySelectorAll<HTMLButtonElement>(".session-line")[1]?.click();
    expect(player.currentTime).toBe(5);
    expect(root.querySelector(".session-line-now .session-line-text")?.textContent).toBe("Mañana envío el informe");

    player.currentTime = 10;
    player.dispatchEvent(new Event("timeupdate"));
    expect(root.querySelector(".session-line-now .session-line-text")?.textContent).toBe("Perfecto, gracias");
  });

  it("keeps the same player when the island refreshes, so playback goes on", async () => {
    await controller.openSession(2);
    const player = root.querySelector("video.session-player");
    await controller.refresh();
    expect(root.querySelector("video.session-player")).toBe(player);
  });

  it("finds what was said and opens the session at that moment", async () => {
    controller.selectTab("sessions");
    const search = root.querySelector<HTMLInputElement>("#sessions-search");
    if (!search) {
      throw new Error("no search box");
    }
    search.value = "informe";
    search.dispatchEvent(new Event("input"));
    await vi.waitFor(() => expect(root.querySelector(".session-hit")).not.toBeNull());
    expect(root.querySelector(".session-hit .island-row-text")?.firstChild?.textContent).toBe("Mañana envío el informe");

    root.querySelector<HTMLButtonElement>(".session-hit .island-row-text")?.click();
    await vi.waitFor(() => expect(root.querySelector(".session-detail")).not.toBeNull());
    expect(root.querySelector<HTMLVideoElement>("video.session-player")?.currentTime).toBe(5);
  });

  it("exports a session to the user's Downloads", async () => {
    await controller.openSession(2);
    root.querySelector<HTMLButtonElement>(".session-export")?.click();
    await vi.waitFor(() => expect(backend.exportedSessions).toEqual([2]));
  });

  it("deletes a session only after a second press", async () => {
    await controller.openSession(2);
    root.querySelector<HTMLButtonElement>(".session-delete")?.click();
    expect(backend.deletedSessions).toEqual([]);
    expect(root.querySelector(".session-delete")?.textContent).toBe("¿Borrar? Pulsa otra vez");
    root.querySelector<HTMLButtonElement>(".session-delete")?.click();
    await vi.waitFor(() => expect(backend.deletedSessions).toEqual([2]));
    expect(controller.currentState.panel).toBe("tabs");
  });

  it("keeps only the transcript of a session without an AI key", async () => {
    backend.reportTranscriptReady(2);
    await controller.openSession(2);
    expect(backend.savedSessionNotes.size).toBe(0);
    expect(root.querySelector(".session-notes .session-note")?.textContent).toBe("Con una clave de IA, Tiki escribe el resumen, las decisiones y las tareas de cada sesión.");
  });

  it("says how far the transcription model download got", async () => {
    controller.selectTab("sessions");
    backend.reportModelDownload({ downloadedBytes: 35, totalBytes: 100 });
    expect(root.querySelector(".sessions-download")?.textContent).toBe("Preparando la transcripción: 35\u00a0%");
    backend.reportModelDownload({ downloadedBytes: 100, totalBytes: 100 });
    expect(root.querySelector<HTMLElement>(".sessions-download")?.hidden).toBe(true);
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

  const sessionNotes = {
    summary: "Se revisó la facturación de octubre.",
    keyPoints: ["Octubre cerrado"],
    decisions: ["Enviar el informe mañana"],
    tasks: [{ title: "Enviar el informe", dueDate: "2026-10-22" }],
  };

  it("writes a session's notes as soon as its transcript is ready", async () => {
    backend.aiShell = new FakeShell(claudeReads(sessionNotes));
    backend.reportTranscriptReady(2);
    await vi.waitFor(() => expect(backend.savedSessionNotes.get(2)?.summary).toBe("Se revisó la facturación de octubre."));
    expect(JSON.stringify(backend.aiShell.lastBody())).toContain("Mañana envío el informe");
  });

  it("does not write notes by itself when the user turned that off, but writes them on request", async () => {
    await controller.changePreferences({ ...backend.savedPreferences, sessionNotesAutomatic: false });
    backend.aiShell = new FakeShell(claudeReads(sessionNotes));
    backend.reportTranscriptReady(2);
    await controller.refresh();
    expect(backend.savedSessionNotes.size).toBe(0);

    await controller.openSession(2);
    root.querySelector<HTMLButtonElement>(".session-write-notes")?.click();
    await vi.waitFor(() => expect(backend.savedSessionNotes.has(2)).toBe(true));
  });

  it("shows the notes and creates a proposed task with one click", async () => {
    await backend.saveSessionNotes(2, sessionNotes);
    await controller.openSession(2);
    expect(root.querySelector(".session-summary")?.textContent).toBe("Se revisó la facturación de octubre.");
    expect([...root.querySelectorAll(".session-notes h4")].map((title) => title.textContent)).toEqual(["Puntos clave", "Decisiones", "Tareas propuestas"]);

    root.querySelector<HTMLButtonElement>(".session-task-apply")?.click();
    await vi.waitFor(() => expect(root.querySelector(".session-task-created")).not.toBeNull());
    expect(backend.appliedSessionTasks).toEqual([1]);
  });

  it("fills the fields of an item with AI and saves them on the second Enter, learning from changes", async () => {
    backend.aiShell = new FakeShell(captureReply(item({
      title: "Revisar acceso de facturas",
      projectName: "atlas",
      dueDate: "2026-10-23",
      priority: "high",
      remindAt: "2026-10-23T09:00",
      tags: ["accesos"],
    })));
    await write("revisar acceso facturas atlas viernes, importante, avísame a las 9");
    expect(root.querySelector<HTMLInputElement>("#capture-input")?.value).toBe("Revisar acceso de facturas");
    expect(root.querySelector("#capture-project")?.textContent).toBe("Atlas");
    expect(root.querySelector<HTMLInputElement>("#capture-due")?.value).toBe("2026-10-23");
    expect(root.querySelector<HTMLSelectElement>("#capture-priority")?.value).toBe("high");
    expect(backend.confirmedCaptures).toHaveLength(0);

    const due = root.querySelector<HTMLInputElement>("#capture-due");
    if (due) {
      due.value = "2026-10-24";
    }
    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ title: "Revisar acceso de facturas", projectName: "Atlas", dueDate: "2026-10-24" });
    await vi.waitFor(() => expect(backend.lessons).toHaveLength(1));
    expect(backend.lessons[0]).toMatchObject({ understood: { dueDate: "2026-10-23" }, corrected: { dueDate: "2026-10-24" } });
    expect(controller.currentState.panel).toBe("tabs");
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
    backend.aiShell = new FakeShell(answerReply("Con Acme tienes el tracking de formularios para mañana."));
    await write("¿qué tengo con Acme?");

    expect(root.querySelector(".capture-answer .ask-answer")?.textContent).toBe("Con Acme tienes el tracking de formularios para mañana.");
    expect(root.querySelector<HTMLElement>(".capture-fields")?.hidden).toBe(true);
    expect(backend.confirmedCaptures).toHaveLength(0);
    const sent = JSON.stringify(backend.aiShell.lastBody().messages);
    expect(sent).toContain("Tracking de formularios");
    expect(sent).toContain("¿qué tengo con Acme?");
  });

  it("applies the actions Tiki proposes only when the user clicks", async () => {
    backend.aiShell = new FakeShell(answerReply("Listo, te propongo esto.", [
      { type: "set_due_date", summary: "Tracking vence el viernes", taskId: 1, title: null, projectName: null, dueDate: "2026-10-23", remindAt: null, status: null },
      { type: "create_reminder", summary: "Avisarte mañana a las 10", taskId: null, title: "Llamar a Lucía", projectName: "Acme", dueDate: null, remindAt: "2026-10-22T10:00", status: null },
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
    await write("llamar a acme");
    expect(root.querySelector(".capture-status")?.textContent).toContain("No pude");
    expect(backend.confirmedCaptures).toHaveLength(0);

    await controller.submitCapture();
    expect(backend.confirmedCaptures[0]).toMatchObject({ title: "llamar a acme", projectName: null, kind: "task" });
  });

  it("writes the day's AI brief in the summary card of Today", async () => {
    backend.aiShell = new FakeShell(claudeReads({ text: "Empieza por el tracking de Acme." }));
    controller.setMode("expanded");
    root.querySelector<HTMLButtonElement>("#prepare-brief")?.click();
    await vi.waitFor(() => expect(controller.currentState.dayBrief).toBe("Empieza por el tracking de Acme."));
    expect(root.querySelector("#today-summary")?.textContent).toBe("Empieza por el tracking de Acme.");
  });

});
