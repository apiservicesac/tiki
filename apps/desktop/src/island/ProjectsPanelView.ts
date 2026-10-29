import type { ProjectChange, ProjectSummary } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";
import type { Translator } from "../i18n/Translator";

export interface ProjectsPanelHandlers {
  onCreateProject: (name: string) => void;
  onChangeProject: (projectId: number, change: ProjectChange) => void;
  onRemoveProject: (projectId: number) => void;
}

/**
 * Every project, with one box on top: typing filters the list, and a name no project has
 * yet can be created from it (button or Enter). Each project can be renamed, recolored or
 * removed. Built once and kept, so the search and the project being edited survive refreshes.
 */
export class ProjectsPanelView {
  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private list: HTMLElement | null = null;
  private editing: { projectId: number; name: string; color: string; notes: string } | null = null;
  private confirmingRemoval: number | null = null;
  private projects: ProjectSummary[] = [];
  private colors: string[] = [];
  private handlers: ProjectsPanelHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(projects: ProjectSummary[], colors: string[], handlers: ProjectsPanelHandlers): HTMLElement {
    this.projects = projects;
    this.colors = colors;
    this.handlers = handlers;
    const firstBuild = this.element === null;
    this.element ??= this.build();
    this.drawList();
    if (firstBuild) {
      // Creating a project and searching start in the same box.
      queueMicrotask(() => this.search?.focus());
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
        id: "projects-search",
        autocomplete: "off",
        placeholder: this.translator.text("projects.search_or_create"),
        "aria-label": this.translator.text("projects.search_or_create"),
      },
    });
    this.search.addEventListener("input", () => this.drawList());
    this.search.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        this.createTyped();
      }
    });
    this.list = ElementFactory.create("div", { className: "island-list projects-list" });
    return ElementFactory.create("div", { className: "projects-panel", children: [this.search, this.list] });
  }

  /** The name typed in the box when no project is called that yet. */
  private newName(): string | null {
    const name = this.search?.value.trim() ?? "";
    const taken = this.projects.some((project) => project.name.toLocaleLowerCase() === name.toLocaleLowerCase());
    return name === "" || taken ? null : name;
  }

  private createTyped(): void {
    const name = this.newName();
    if (name && this.search) {
      this.search.value = "";
      this.handlers?.onCreateProject(name);
    }
  }

  /** "Create «Northern Builders»", first in the list while the typed name is new. */
  private createRow(name: string): HTMLElement {
    return ElementFactory.textButton({
      className: "projects-create",
      text: this.translator.format("projects.create", { name }),
      attributes: { id: "add-project" },
    }, () => this.createTyped());
  }

  private drawList(): void {
    if (!this.list) {
      return;
    }
    const query = this.search?.value ?? "";
    const projects = this.projects.filter((project) => TextMatcher.matches(project.name, query));
    const newName = this.newName();
    const rows = projects.map((project) => (this.editing?.projectId === project.projectId ? this.editRow(project) : this.row(project)));
    const empty = rows.length === 0 && !newName
      ? [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("projects.empty") })]
      : [];
    this.list.replaceChildren(...(newName ? [this.createRow(newName)] : []), ...rows, ...empty);
  }

  private row(project: ProjectSummary): HTMLElement {
    const dot = ElementFactory.create("span", { className: "picker-dot" });
    dot.style.background = project.color;
    const confirming = this.confirmingRemoval === project.projectId;
    const remove = ElementFactory.textButton({
      className: confirming ? "project-remove project-remove-confirm" : "island-icon-button project-remove",
      attributes: { "aria-label": this.translator.text("projects.remove"), title: this.translator.text("projects.remove") },
    }, () => {
      if (confirming) {
        this.confirmingRemoval = null;
        this.handlers?.onRemoveProject(project.projectId);
      } else {
        this.confirmingRemoval = project.projectId;
        this.drawList();
      }
    });
    if (confirming) {
      remove.textContent = this.translator.text("projects.remove_confirm");
    } else {
      remove.innerHTML = ICONS.trash;
    }
    const edit = ElementFactory.iconButton(ICONS.pencil, this.translator.text("projects.edit"), () => {
      this.editing = { projectId: project.projectId, name: project.name, color: project.color, notes: project.notes };
      this.confirmingRemoval = null;
      this.drawList();
    });
    return ElementFactory.create("div", {
      className: "project-row",
      attributes: { "data-project-id": String(project.projectId) },
      children: [
        dot,
        ElementFactory.create("span", {
          className: "project-name",
          text: project.name,
          children: [ElementFactory.create("small", { text: this.workload(project) })],
        }),
        edit,
        remove,
      ],
    });
  }

  private editRow(project: ProjectSummary): HTMLElement {
    const editing = this.editing ?? { projectId: project.projectId, name: project.name, color: project.color, notes: project.notes };
    const notes = ElementFactory.create("textarea", {
      className: "edit-notes",
      attributes: { id: "project-notes-input", rows: "3", placeholder: this.translator.text("projects.notes_placeholder") },
    });
    notes.value = editing.notes;
    notes.addEventListener("input", () => {
      if (this.editing) {
        this.editing.notes = notes.value;
      }
    });
    const name = ElementFactory.create("input", {
      className: "settings-input",
      attributes: { type: "text", id: "project-name-input", autocomplete: "off", "aria-label": this.translator.text("projects.name") },
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
        this.handlers?.onChangeProject(project.projectId, { name: name.value.trim(), color: editing.color, notes: notes.value.trim() });
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
      className: "project-row project-row-editing",
      children: [
        name,
        ElementFactory.create("div", { className: "project-palette", children: this.swatches(editing.color) }),
        notes,
        ElementFactory.create("div", { className: "edit-actions", children: [this.cancelButton(), this.saveButton(save)] }),
      ],
    });
  }

  private swatches(selected: string): HTMLElement[] {
    return this.colors.map((color) => {
      const swatch = ElementFactory.textButton({
        className: color === selected ? "project-swatch project-swatch-on" : "project-swatch",
        attributes: { "aria-label": color, "data-color": color },
      }, () => {
        if (this.editing) {
          this.editing.color = color;
          this.drawList();
        }
      });
      swatch.style.background = color;
      return swatch;
    });
  }

  private saveButton(save: () => void): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-button",
      text: this.translator.text("edit.save"),
      attributes: { id: "save-project" },
    }, save);
    return button;
  }

  private cancelButton(): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-button settings-button-quiet",
      text: this.translator.text("edit.cancel"),
    }, () => {
      this.editing = null;
      this.drawList();
    });
    return button;
  }

  private workload(project: ProjectSummary): string {
    if (project.openCount === 0) {
      return this.translator.text("projects.no_open");
    }
    return project.openCount === 1
      ? this.translator.text("projects.open_one")
      : this.translator.format("projects.open", { count: project.openCount });
  }
}
