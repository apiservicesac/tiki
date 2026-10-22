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
 * Every project, with a search box: create one, rename it, change its color or remove it.
 * Built once and kept, so the search and the project being edited survive refreshes.
 */
export class ProjectsPanelView {
  private element: HTMLElement | null = null;
  private search: HTMLInputElement | null = null;
  private newProject: HTMLInputElement | null = null;
  private list: HTMLElement | null = null;
  private editing: { projectId: number; name: string; color: string; notes: string } | null = null;
  private confirmingRemoval: number | null = null;
  private projects: ProjectSummary[] = [];
  private colors: string[] = [];
  private handlers: ProjectsPanelHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(projects: ProjectSummary[], colors: string[], focusNewProject: boolean, handlers: ProjectsPanelHandlers): HTMLElement {
    this.projects = projects;
    this.colors = colors;
    this.handlers = handlers;
    const firstBuild = this.element === null;
    this.element ??= this.build();
    this.drawList();
    if (firstBuild) {
      queueMicrotask(() => (focusNewProject ? this.newProject : this.search)?.focus());
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
        placeholder: this.translator.text("projects.search"),
        "aria-label": this.translator.text("projects.search"),
      },
    });
    this.search.addEventListener("input", () => this.drawList());
    this.newProject = ElementFactory.create("input", {
      className: "settings-input",
      attributes: {
        type: "text",
        id: "new-project",
        autocomplete: "off",
        placeholder: this.translator.text("projects.new_placeholder"),
        "aria-label": this.translator.text("projects.new_placeholder"),
      },
    });
    const add = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("projects.add"),
      attributes: { type: "button", id: "add-project" },
    });
    const create = () => {
      const name = this.newProject?.value.trim() ?? "";
      if (name !== "" && this.newProject) {
        this.newProject.value = "";
        this.handlers?.onCreateProject(name);
      }
    };
    add.addEventListener("click", (event) => {
      event.stopPropagation();
      create();
    });
    this.newProject.addEventListener("keydown", (event) => {
      if (event.key === "Enter") {
        create();
      }
    });
    this.list = ElementFactory.create("div", { className: "island-list projects-list" });
    return ElementFactory.create("div", {
      className: "projects-panel",
      children: [
        ElementFactory.create("div", { className: "settings-row projects-new", children: [this.newProject, add] }),
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
    const projects = this.projects.filter((project) => TextMatcher.matches(project.name, query));
    this.list.replaceChildren(
      ...(projects.length > 0
        ? projects.map((project) => (this.editing?.projectId === project.projectId ? this.editRow(project) : this.row(project)))
        : [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("projects.empty") })]),
    );
  }

  private row(project: ProjectSummary): HTMLElement {
    const dot = ElementFactory.create("span", { className: "picker-dot" });
    dot.style.background = project.color;
    const confirming = this.confirmingRemoval === project.projectId;
    const remove = ElementFactory.create("button", {
      className: confirming ? "project-remove project-remove-confirm" : "island-icon-button project-remove",
      attributes: { type: "button", "aria-label": this.translator.text("projects.remove"), title: this.translator.text("projects.remove") },
    });
    if (confirming) {
      remove.textContent = this.translator.text("projects.remove_confirm");
    } else {
      remove.innerHTML = ICONS.trash;
    }
    remove.addEventListener("click", (event) => {
      event.stopPropagation();
      if (confirming) {
        this.confirmingRemoval = null;
        this.handlers?.onRemoveProject(project.projectId);
      } else {
        this.confirmingRemoval = project.projectId;
        this.drawList();
      }
    });
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
      const swatch = ElementFactory.create("button", {
        className: color === selected ? "project-swatch project-swatch-on" : "project-swatch",
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
      attributes: { type: "button", id: "save-project" },
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

  private workload(project: ProjectSummary): string {
    if (project.openCount === 0) {
      return this.translator.text("projects.no_open");
    }
    return this.translator.format("projects.open", { count: project.openCount });
  }
}
