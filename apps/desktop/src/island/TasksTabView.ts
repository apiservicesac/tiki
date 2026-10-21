import type { ClientSummary, DeliveredItem, SnapshotItem, SnapshotSection, Urgency } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";
import type { Translator } from "../i18n/Translator";
import { ClientPicker } from "./ClientPicker";
import type { TaskListHandlers, TaskListView } from "./TaskListView";

export type TasksView = "pending" | "delivered";

export interface TasksTabData {
  sections: SnapshotSection[];
  delivered: DeliveredItem[];
  clients: ClientSummary[];
  view: TasksView;
}

export interface TasksTabHandlers extends TaskListHandlers {
  onSelectTasksView: (view: TasksView) => void;
  /** Opens the client panel; `createNew` focuses the new-client box. */
  onManageClients: (createNew: boolean) => void;
}

interface ClientGroup {
  name: string | null;
  color: string | null;
  items: SnapshotItem[];
  /** The most pressing urgency in the group: groups with overdue work come first. */
  rank: number;
}

/**
 * The Tasks tab: every task, grouped by client, with a search box, a client picker and the
 * pending or delivered view. Built once and kept, so typing in the search box survives the
 * island's refreshes; only the list is redrawn.
 */
export class TasksTabView {
  private static readonly URGENCY_RANK: Readonly<Record<Urgency, number>> = Object.freeze({
    critical: 0,
    soon: 1,
    waiting: 2,
    normal: 3,
  });

  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private toggle: HTMLElement | null = null;
  private list: HTMLElement | null = null;
  private readonly picker = new ClientPicker();
  private clientFilter: string | null = null;
  private data: TasksTabData | null = null;
  private handlers: TasksTabHandlers | null = null;

  constructor(
    private readonly translator: Translator,
    private readonly taskList: TaskListView,
  ) {}

  render(data: TasksTabData, handlers: TasksTabHandlers): HTMLElement {
    this.data = data;
    this.handlers = handlers;
    if (this.clientFilter !== null && !data.clients.some((client) => client.name === this.clientFilter)) {
      this.clientFilter = null;
    }
    this.element ??= this.build();
    this.redraw();
    return this.element;
  }

  /** Forgets the tab, so it is rebuilt in a new language. */
  reset(): void {
    this.element = null;
    this.clientFilter = null;
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
      id: "tasks-client-picker",
      options: [
        { value: null, label: this.translator.text("tasks.all_clients"), color: null },
        ...data.clients.map((client) => ({ value: client.name, label: client.name, color: client.color, count: client.openCount })),
      ],
      selected: this.clientFilter,
      searchPlaceholder: this.translator.text("tasks.search_client"),
      emptyText: this.translator.text("tasks.no_client_match"),
      actions: [
        { label: this.translator.text("tasks.new_client"), onPress: () => this.handlers?.onManageClients(true) },
        { label: this.translator.text("tasks.manage_clients"), onPress: () => this.handlers?.onManageClients(false) },
      ],
      onSelect: (value) => {
        this.clientFilter = value;
        this.redraw();
      },
    });
  }

  private drawToggle(): void {
    const view = this.data?.view ?? "pending";
    const buttons = (["pending", "delivered"] as const).map((option) => {
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
    const rows = data.view === "pending"
      ? this.pendingRows(data.sections, query, handlers)
      : this.deliveredRows(data.delivered, query, handlers);
    const emptyKey = data.view === "pending" ? "tasks.empty_pending" : "tasks.empty_delivered";
    this.list.replaceChildren(
      ...(rows.length > 0 ? rows : [ElementFactory.create("p", { className: "island-empty", text: this.translator.text(emptyKey) })]),
    );
  }

  private pendingRows(sections: SnapshotSection[], query: string, handlers: TaskListHandlers): HTMLElement[] {
    const items = sections
      .flatMap((section) => section.items)
      .filter((item) => this.keeps(item.title, item.clientName, query));
    return this.groups(items).flatMap((group) => [
      this.groupHeader(group),
      ...group.items.map((item) => this.taskList.row(item, handlers)),
    ]);
  }

  private deliveredRows(delivered: DeliveredItem[], query: string, handlers: TaskListHandlers): HTMLElement[] {
    return delivered
      .filter((item) => this.keeps(item.title, item.clientName, query))
      .map((item) => this.taskList.deliveredRow(item, handlers.onOpenTask));
  }

  /** Whether a task passes the client filter and the search (title or client name). */
  private keeps(title: string, clientName: string | null, query: string): boolean {
    if (this.clientFilter !== null && clientName !== this.clientFilter) {
      return false;
    }
    return TextMatcher.matches(title, query) || (clientName !== null && TextMatcher.matches(clientName, query));
  }

  /** One group per client: the most pressing first, then the busiest, tasks without client last. */
  private groups(items: SnapshotItem[]): ClientGroup[] {
    const groups = new Map<string, ClientGroup>();
    items.forEach((item) => {
      const key = item.clientName ?? "";
      const group = groups.get(key) ?? { name: item.clientName, color: item.clientColor, items: [], rank: Number.MAX_SAFE_INTEGER };
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

  private groupHeader(group: ClientGroup): HTMLElement {
    const dot = ElementFactory.create("i", { className: "tasks-group-dot" });
    dot.style.background = group.color ?? "var(--island-faint)";
    return ElementFactory.create("div", {
      className: "island-group tasks-group",
      children: [
        dot,
        ElementFactory.create("span", { text: group.name ?? this.translator.text("edit.no_client") }),
        ElementFactory.create("span", { className: "tasks-group-count", text: String(group.items.length) }),
      ],
    });
  }
}
