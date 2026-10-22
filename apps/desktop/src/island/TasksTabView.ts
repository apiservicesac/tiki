import type { ProjectSummary, DeliveredItem, SnapshotItem, Urgency } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";
import type { Translator } from "../i18n/Translator";
import { ProjectPicker } from "./ProjectPicker";
import type { TaskListHandlers, TaskListView } from "./TaskListView";

/** Pending tasks, or the delivered history. Reminders live in the Agenda. */
export const TASKS_VIEWS = Object.freeze(["pending", "delivered"] as const);
export type TasksView = (typeof TASKS_VIEWS)[number];

export interface TasksTabData {
  /** Every open item. */
  items: SnapshotItem[];
  delivered: DeliveredItem[];
  projects: ProjectSummary[];
  view: TasksView;
}

export interface TasksTabHandlers extends TaskListHandlers {
  onSelectTasksView: (view: TasksView) => void;
  /** Opens the project panel; `createNew` focuses the new-project box. */
  onManageProjects: (createNew: boolean) => void;
}

interface ProjectGroup {
  name: string | null;
  color: string | null;
  items: SnapshotItem[];
  /** The most pressing urgency in the group: groups with overdue work come first. */
  rank: number;
}

/**
 * The Tasks tab: every task, grouped by project, with a search box, a project picker and the
 * pending or delivered view. Built once and kept, so typing in the search box survives the
 * island's refreshes; only the list is redrawn.
 */
export class TasksTabView {
  private static readonly URGENCY_RANK: Readonly<Record<Urgency, number>> = Object.freeze({
    critical: 0,
    soon: 1,
    waiting: 2,
    normal: 3,
    reminder: 4,
  });

  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private toggle: HTMLElement | null = null;
  private list: HTMLElement | null = null;
  private readonly picker = new ProjectPicker();
  private projectFilter: string | null = null;
  private data: TasksTabData | null = null;
  private handlers: TasksTabHandlers | null = null;

  constructor(
    private readonly translator: Translator,
    private readonly taskList: TaskListView,
  ) {}

  render(data: TasksTabData, handlers: TasksTabHandlers): HTMLElement {
    this.data = data;
    this.handlers = handlers;
    if (this.projectFilter !== null && !data.projects.some((project) => project.name === this.projectFilter)) {
      this.projectFilter = null;
    }
    this.element ??= this.build();
    this.redraw();
    return this.element;
  }

  /** Forgets the tab, so it is rebuilt in a new language. */
  reset(): void {
    this.element = null;
    this.projectFilter = null;
  }

  private build(): HTMLElement {
    this.search = ElementFactory.create("input", {
      className: "settings-input tasks-search",
      attributes: {
        type: "search",
        id: "tasks-search",
        autocomplete: "off",
        spellcheck: "false",
        placeholder: this.translator.text("tasks.search"),
        "aria-label": this.translator.text("tasks.search"),
      },
    });
    this.search.addEventListener("input", () => this.drawList());
    this.toggle = ElementFactory.create("div", { className: "tasks-toggle", attributes: { role: "tablist" } });
    this.list = ElementFactory.create("div", { className: "island-list tasks-list" });
    return ElementFactory.create("div", {
      className: "tasks-tab",
      children: [
        ElementFactory.create("div", {
          className: "tasks-toolbar",
          children: [
            ElementFactory.create("label", {
              className: "tasks-search-box",
              children: [ElementFactory.fromTrustedMarkup("tasks-search-icon", ICONS.search), this.search],
            }),
            this.picker.element,
            this.toggle,
          ],
        }),
        this.list,
      ],
    });
  }

  private redraw(): void {
    this.drawPicker();
    this.drawToggle();
    this.drawList();
  }

  private drawPicker(): void {
    const data = this.data;
    if (!data) {
      return;
    }
    this.picker.update({
      id: "tasks-project-picker",
      options: [
        { value: null, label: this.translator.text("tasks.all_projects"), color: null },
        ...data.projects.map((project) => ({ value: project.name, label: project.name, color: project.color, count: project.openCount })),
      ],
      selected: this.projectFilter,
      searchPlaceholder: this.translator.text("tasks.search_project"),
      emptyText: this.translator.text("tasks.no_project_match"),
      actions: [
        { label: this.translator.text("tasks.new_project"), onPress: () => this.handlers?.onManageProjects(true) },
        { label: this.translator.text("tasks.manage_projects"), onPress: () => this.handlers?.onManageProjects(false) },
      ],
      onSelect: (value) => {
        this.projectFilter = value;
        this.redraw();
      },
    });
  }

  private drawToggle(): void {
    const view = this.data?.view ?? "pending";
    const buttons = TASKS_VIEWS.map((option) => {
      const button = ElementFactory.create("button", {
        className: option === view ? "tasks-toggle-option tasks-toggle-on" : "tasks-toggle-option",
        text: this.translator.text(`tasks.view.${option}`),
        attributes: { type: "button", role: "tab", "aria-selected": String(option === view), "data-view": option },
      });
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        this.handlers?.onSelectTasksView(option);
      });
      return button;
    });
    this.toggle?.replaceChildren(...buttons);
  }

  private drawList(): void {
    const data = this.data;
    const handlers = this.handlers;
    if (!data || !handlers || !this.list) {
      return;
    }
    const query = this.search?.value ?? "";
    const rows = this.rows(data, query, handlers);
    const emptyKey = `tasks.empty_${data.view}`;
    this.list.replaceChildren(
      ...(rows.length > 0 ? rows : [ElementFactory.create("p", { className: "island-empty", text: this.translator.text(emptyKey) })]),
    );
  }

  private rows(data: TasksTabData, query: string, handlers: TaskListHandlers): HTMLElement[] {
    switch (data.view) {
      case "pending":
        return this.pendingRows(data.items, query, handlers);
      case "delivered":
        return this.deliveredRows(data.delivered, query, handlers);
    }
  }

  /** Tasks only, grouped by project; reminders are in the Agenda. */
  private pendingRows(openItems: SnapshotItem[], query: string, handlers: TaskListHandlers): HTMLElement[] {
    const items = openItems.filter((item) =>
      item.urgency !== "reminder" && this.keeps(item.title, item.projectName, item.tags, query));
    return this.groups(items).flatMap((group) => [
      this.groupHeader(group),
      ...group.items.map((item) => this.taskList.row(item, handlers)),
    ]);
  }

  private deliveredRows(delivered: DeliveredItem[], query: string, handlers: TaskListHandlers): HTMLElement[] {
    return delivered
      .filter((item) => this.keeps(item.title, item.projectName, [], query))
      .map((item) => this.taskList.deliveredRow(item, handlers.onOpenTask));
  }

  /** Whether a task passes the project filter and the search (title, project or #tag). */
  private keeps(title: string, projectName: string | null, tags: readonly string[], query: string): boolean {
    if (this.projectFilter !== null && projectName !== this.projectFilter) {
      return false;
    }
    return TextMatcher.matches(title, query)
      || (projectName !== null && TextMatcher.matches(projectName, query))
      || tags.some((tag) => TextMatcher.matches(`#${tag}`, query));
  }

  /** One group per project: the most pressing first, then the busiest, tasks without project last. */
  private groups(items: SnapshotItem[]): ProjectGroup[] {
    const groups = new Map<string, ProjectGroup>();
    items.forEach((item) => {
      const key = item.projectName ?? "";
      const group = groups.get(key) ?? { name: item.projectName, color: item.projectColor, items: [], rank: Number.MAX_SAFE_INTEGER };
      group.items.push(item);
      group.rank = Math.min(group.rank, TasksTabView.URGENCY_RANK[item.urgency]);
      groups.set(key, group);
    });
    return [...groups.values()].sort((first, second) => {
      if ((first.name === null) !== (second.name === null)) {
        return first.name === null ? 1 : -1;
      }
      return first.rank - second.rank
        || second.items.length - first.items.length
        || (first.name ?? "").localeCompare(second.name ?? "");
    });
  }

  private groupHeader(group: ProjectGroup): HTMLElement {
    const dot = ElementFactory.create("i", { className: "tasks-group-dot" });
    dot.style.background = group.color ?? "var(--island-faint)";
    return ElementFactory.create("div", {
      className: "island-group tasks-group",
      children: [
        dot,
        ElementFactory.create("span", { text: group.name ?? this.translator.text("edit.no_project") }),
        ElementFactory.create("span", { className: "tasks-group-count", text: String(group.items.length) }),
      ],
    });
  }
}
