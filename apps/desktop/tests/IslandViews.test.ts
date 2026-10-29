import { beforeEach, describe, expect, it, vi } from "vitest";
import type { TikiAlert } from "../src/backend/contracts";
import { Translator } from "../src/i18n/Translator";
import { AgendaView, type AgendaHandlers } from "../src/island/AgendaView";
import { TaskListView } from "../src/island/TaskListView";
import { TasksTabView, type TasksTabHandlers } from "../src/island/TasksTabView";
import { TodayView, type TodayHandlers } from "../src/island/TodayView";
import { FakeBackend } from "./support/FakeBackend";

const SNAPSHOT = FakeBackend.sampleSnapshot();
const DELIVERED = new FakeBackend().delivered;
const TODAY = new Date(2026, 9, 23, 8, 0);

const REMINDER_ALERT: TikiAlert = {
  kind: "reminder",
  title: "Tomar agua",
  body: "Son las 11:00",
  taskId: 8,
  link: null,
};

/** Every handler a view can call, recording the calls. */
function recordingHandlers(): TodayHandlers & TasksTabHandlers & AgendaHandlers {
  return {
    onDeliverTask: vi.fn(),
    onOpenTask: vi.fn(),
    onOpenAlert: vi.fn(),
    onDismissAlert: vi.fn(),
    onSnooze: vi.fn(),
    onJoinMeeting: vi.fn(),
    onPrepareBrief: vi.fn(),
    onShowDelivered: vi.fn(),
    onSelectTasksView: vi.fn(),
    onManageProjects: vi.fn(),
  };
}

/** Puts a view inside an element that records clicks bubbling out of it. */
function mount(view: HTMLElement): { bubbled: () => number } {
  let clicks = 0;
  const island = document.createElement("div");
  island.addEventListener("click", () => {
    clicks += 1;
  });
  island.append(view);
  document.body.replaceChildren(island);
  return { bubbled: () => clicks };
}

function click(root: ParentNode, selector: string): void {
  const element = root.querySelector<HTMLElement>(selector);
  expect(element, selector).not.toBeNull();
  element?.click();
}

describe("TodayView", () => {
  const translator = new Translator("es");
  const view = new TodayView(translator, new TaskListView(translator));

  it("draws the focus, today's rows and the delivered link", () => {
    const element = view.render(
      { snapshot: SNAPSHOT, alert: null, dayBrief: null, preparingBrief: false, aiReady: true },
      recordingHandlers(),
    );
    expect(element.outerHTML).toMatchSnapshot();
  });

  it("draws the latest reminder with its buttons", () => {
    const element = view.render(
      { snapshot: SNAPSHOT, alert: REMINDER_ALERT, dayBrief: null, preparingBrief: false, aiReady: true },
      recordingHandlers(),
    );
    expect(element.querySelector(".island-notice")?.outerHTML).toMatchSnapshot();
  });

  it("calls its handlers without letting the click reach the island", () => {
    const handlers = recordingHandlers();
    const element = view.render(
      { snapshot: SNAPSHOT, alert: REMINDER_ALERT, dayBrief: null, preparingBrief: false, aiReady: true },
      handlers,
    );
    const island = mount(element);
    click(element, "#today-focus");
    click(element, '.island-row[data-task-id="1"] .island-row-text');
    click(element, '.island-row[data-task-id="1"] .island-row-done');
    click(element, "#alert-done");
    click(element, "#alert-snooze-10");
    click(element, "#delivered-today");
    expect(handlers.onOpenTask).toHaveBeenNthCalledWith(1, 3);
    expect(handlers.onOpenTask).toHaveBeenNthCalledWith(2, 1);
    expect(handlers.onDeliverTask).toHaveBeenCalledWith(1);
    expect(handlers.onDismissAlert).toHaveBeenCalledOnce();
    expect(handlers.onSnooze).toHaveBeenCalledWith(8, 10);
    expect(handlers.onShowDelivered).toHaveBeenCalledOnce();
    expect(island.bubbled()).toBe(0);
  });
});

describe("TasksTabView", () => {
  let view: TasksTabView;

  beforeEach(() => {
    const translator = new Translator("es");
    view = new TasksTabView(translator, new TaskListView(translator));
  });

  const data = (tasksView: "pending" | "delivered") => ({
    items: SNAPSHOT.openItems,
    delivered: DELIVERED,
    projects: [],
    view: tasksView,
  });

  it("draws pending tasks grouped by project", () => {
    const element = view.render(data("pending"), recordingHandlers());
    expect(element.querySelector(".tasks-list")?.outerHTML).toMatchSnapshot();
  });

  it("draws the delivered history", () => {
    const element = view.render(data("delivered"), recordingHandlers());
    expect(element.querySelector(".tasks-list")?.outerHTML).toMatchSnapshot();
  });

  it("opens, delivers and reopens tasks without letting the click reach the island", () => {
    const handlers = recordingHandlers();
    const pending = view.render(data("pending"), handlers);
    const island = mount(pending);
    click(pending, '.island-row[data-task-id="2"] .island-row-text');
    click(pending, '.island-row[data-task-id="2"] .island-row-done');
    click(pending, '[data-view="delivered"]');
    const delivered = view.render(data("delivered"), handlers);
    click(delivered, '.island-row[data-task-id="9"] .island-row-text');
    expect(handlers.onOpenTask).toHaveBeenNthCalledWith(1, 2);
    expect(handlers.onDeliverTask).toHaveBeenCalledWith(2);
    expect(handlers.onSelectTasksView).toHaveBeenCalledWith("delivered");
    expect(handlers.onOpenTask).toHaveBeenNthCalledWith(2, 9);
    expect(island.bubbled()).toBe(0);
  });
});

describe("AgendaView", () => {
  const view = new AgendaView(new Translator("es"));

  it("draws reminders and meetings by day and time", () => {
    const element = view.render(SNAPSHOT.openItems, SNAPSHOT.meetings, TODAY, recordingHandlers());
    expect(element.outerHTML).toMatchSnapshot();
  });

  it("opens and delivers a reminder and joins a meeting without letting the click reach the island", () => {
    const handlers = recordingHandlers();
    const element = view.render(SNAPSHOT.openItems, SNAPSHOT.meetings, TODAY, handlers);
    const island = mount(element);
    click(element, '.agenda-row[data-task-id="8"] .island-row-text');
    click(element, '.agenda-row[data-task-id="8"] .island-row-done');
    click(element, ".agenda-join");
    expect(handlers.onOpenTask).toHaveBeenCalledWith(8);
    expect(handlers.onDeliverTask).toHaveBeenCalledWith(8);
    expect(handlers.onJoinMeeting).toHaveBeenCalledWith("https://teams.microsoft.com/l/meetup-join/daily");
    expect(island.bubbled()).toBe(0);
  });
});
