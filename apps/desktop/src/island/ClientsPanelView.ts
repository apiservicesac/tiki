import type { ClientChange, ClientSummary } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";
import type { Translator } from "../i18n/Translator";

export interface ClientsPanelHandlers {
  onCreateClient: (name: string) => void;
  onChangeClient: (clientId: number, change: ClientChange) => void;
  onRemoveClient: (clientId: number) => void;
}

/**
 * Every client, with a search box: create one, rename it, change its color or remove it.
 * Built once and kept, so the search and the client being edited survive refreshes.
 */
export class ClientsPanelView {
  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private newClient: HTMLInputElement | null = null;
  private list: HTMLElement | null = null;
  private editing: { clientId: number; name: string; color: string } | null = null;
  private confirmingRemoval: number | null = null;
  private clients: ClientSummary[] = [];
  private colors: string[] = [];
  private handlers: ClientsPanelHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(clients: ClientSummary[], colors: string[], focusNewClient: boolean, handlers: ClientsPanelHandlers): HTMLElement {
    this.clients = clients;
    this.colors = colors;
    this.handlers = handlers;
    const firstBuild = this.element === null;
    this.element ??= this.build();
    this.drawList();
    if (firstBuild) {
      queueMicrotask(() => (focusNewClient ? this.newClient : this.search)?.focus());
    }
    return this.element;
  }

  reset(): void {
    this.element = null;
    this.editing = null;
    this.confirmingRemoval = null;
  }

  private build(): HTMLElement {
    this.search = ElementFactory.create("input", {
      className: "settings-input",
      attributes: {
        type: "search",
        id: "clients-search",
        autocomplete: "off",
        placeholder: this.translator.text("clients.search"),
        "aria-label": this.translator.text("clients.search"),
      },
    });
    this.search.addEventListener("input", () => this.drawList());
    this.newClient = ElementFactory.create("input", {
      className: "settings-input",
      attributes: {
        type: "text",
        id: "new-client",
        autocomplete: "off",
        placeholder: this.translator.text("clients.new_placeholder"),
        "aria-label": this.translator.text("clients.new_placeholder"),
      },
    });
    const add = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("clients.add"),
      attributes: { type: "button", id: "add-client" },
    });
    const create = () => {
      const name = this.newClient?.value.trim() ?? "";
      if (name !== "" && this.newClient) {
        this.newClient.value = "";
        this.handlers?.onCreateClient(name);
      }
    };
    add.addEventListener("click", (event) => {
      event.stopPropagation();
      create();
    });
    this.newClient.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        create();
      }
    });
    this.list = ElementFactory.create("div", { className: "island-list clients-list" });
    return ElementFactory.create("div", {
      className: "clients-panel",
      children: [
        ElementFactory.create("div", { className: "settings-row clients-new", children: [this.newClient, add] }),
        this.search,
        this.list,
      ],
    });
  }

  private drawList(): void {
    if (!this.list) {
      return;
    }
    const query = this.search?.value ?? "";
    const clients = this.clients.filter((client) => TextMatcher.matches(client.name, query));
    this.list.replaceChildren(
      ...(clients.length > 0
        ? clients.map((client) => (this.editing?.clientId === client.clientId ? this.editRow(client) : this.row(client)))
        : [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("clients.empty") })]),
    );
  }

  private row(client: ClientSummary): HTMLElement {
    const dot = ElementFactory.create("span", { className: "picker-dot" });
    dot.style.background = client.color;
    const confirming = this.confirmingRemoval === client.clientId;
    const remove = ElementFactory.create("button", {
      className: confirming ? "client-remove client-remove-confirm" : "island-icon-button client-remove",
      attributes: { type: "button", "aria-label": this.translator.text("clients.remove"), title: this.translator.text("clients.remove") },
    });
    if (confirming) {
      remove.textContent = this.translator.text("clients.remove_confirm");
    } else {
      remove.innerHTML = ICONS.trash;
    }
    remove.addEventListener("click", (event) => {
      event.stopPropagation();
      if (confirming) {
        this.confirmingRemoval = null;
        this.handlers?.onRemoveClient(client.clientId);
      } else {
        this.confirmingRemoval = client.clientId;
        this.drawList();
      }
    });
    const edit = ElementFactory.iconButton(ICONS.pencil, this.translator.text("clients.edit"), () => {
      this.editing = { clientId: client.clientId, name: client.name, color: client.color };
      this.confirmingRemoval = null;
      this.drawList();
    });
    return ElementFactory.create("div", {
      className: "client-row",
      attributes: { "data-client-id": String(client.clientId) },
      children: [
        dot,
        ElementFactory.create("span", {
          className: "client-name",
          text: client.name,
          children: [ElementFactory.create("small", { text: this.workload(client) })],
        }),
        edit,
        remove,
      ],
    });
  }

  private editRow(client: ClientSummary): HTMLElement {
    const editing = this.editing ?? { clientId: client.clientId, name: client.name, color: client.color };
    const name = ElementFactory.create("input", {
      className: "settings-input",
      attributes: { type: "text", id: "client-name-input", autocomplete: "off", "aria-label": this.translator.text("clients.name") },
    });
    name.value = editing.name;
    name.addEventListener("input", () => {
      if (this.editing) {
        this.editing.name = name.value;
      }
    });
    const save = () => {
      if (name.value.trim() !== "") {
        this.editing = null;
        this.handlers?.onChangeClient(client.clientId, { name: name.value.trim(), color: editing.color });
      }
    };
    name.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        save();
      } else if (event.key === "Escape") {
        event.stopPropagation();
        this.editing = null;
        this.drawList();
      }
    });
    queueMicrotask(() => name.focus());
    return ElementFactory.create("div", {
      className: "client-row client-row-editing",
      children: [
        name,
        ElementFactory.create("div", { className: "client-palette", children: this.swatches(editing.color) }),
        ElementFactory.create("div", { className: "edit-actions", children: [this.cancelButton(), this.saveButton(save)] }),
      ],
    });
  }

  private swatches(selected: string): HTMLElement[] {
    return this.colors.map((color) => {
      const swatch = ElementFactory.create("button", {
        className: color === selected ? "client-swatch client-swatch-on" : "client-swatch",
        attributes: { type: "button", "aria-label": color, "data-color": color },
      });
      swatch.style.background = color;
      swatch.addEventListener("click", (event) => {
        event.stopPropagation();
        if (this.editing) {
          this.editing.color = color;
          this.drawList();
        }
      });
      return swatch;
    });
  }

  private saveButton(save: () => void): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("edit.save"),
      attributes: { type: "button", id: "save-client" },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      save();
    });
    return button;
  }

  private cancelButton(): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-button settings-button-quiet",
      text: this.translator.text("edit.cancel"),
      attributes: { type: "button" },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      this.editing = null;
      this.drawList();
    });
    return button;
  }

  private workload(client: ClientSummary): string {
    if (client.openCount === 0) {
      return this.translator.text("clients.no_open");
    }
    return this.translator.format("clients.open", { count: client.openCount });
  }
}
