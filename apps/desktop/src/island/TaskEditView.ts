import { ITEM_KINDS, PRIORITIES, REPEATS, TASK_STATUSES, type ProjectSummary, type TaskDetails, type TaskEdit } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { TagText } from "../dom/TagText";
import type { Translator } from "../i18n/Translator";
import { ProjectPicker } from "./ProjectPicker";

export interface TaskEditHandlers {
  onSave: (taskId: number, edit: TaskEdit) => void;
  onCancel: () => void;
}

interface TaskEditFields {
  title: HTMLInputElement;
  /** The chosen project's name, or `null` for no project; a new name is created on save. */
  project: { value: string | null };
  dueDate: HTMLInputElement;
  remindAt: HTMLInputElement;
  repeat: HTMLSelectElement;
  priority: HTMLSelectElement;
  status: HTMLSelectElement;
  waitingFor: HTMLInputElement;
  kind: HTMLSelectElement;
  tags: HTMLInputElement;
  notes: HTMLTextAreaElement;
}

/**
 * The form to change a saved item. A task shows its work fields (project, due date, status,
 * priority, reminder, tags, and who it waits for while waiting); a reminder shows only its
 * moment and repeat. It is kept while the same item is open, so background refreshes never
 * wipe what the user is changing.
 */
export class TaskEditView {
  private shown: { taskId: number; element: HTMLElement } | null = null;
  private handlers: TaskEditHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(task: TaskDetails, projects: readonly ProjectSummary[], handlers: TaskEditHandlers): HTMLElement {
    this.handlers = handlers;
    if (this.shown?.taskId !== task.taskId) {
      this.shown = { taskId: task.taskId, element: this.build(task, projects) };
    }
    return this.shown.element;
  }

  /** Forgets the form, so the next item opens with fresh values. */
  reset(): void {
    this.shown = null;
  }

  private build(task: TaskDetails, projects: readonly ProjectSummary[]): HTMLElement {
    const project = { value: task.projectName };
    const picker = new ProjectPicker();
    const drawPicker = () => picker.update({
      id: "edit-project",
      options: [
        { value: null, label: this.translator.text("edit.no_project"), color: null },
        ...projects.map((summary) => ({ value: summary.name, label: summary.name, color: summary.color })),
      ],
      selected: project.value,
      searchPlaceholder: this.translator.text("tasks.search_project"),
      emptyText: this.translator.text("tasks.no_project_match"),
      createLabel: (name) => this.translator.format("projects.create", { name }),
      onSelect: (value) => {
        project.value = value;
        drawPicker();
      },
    });
    drawPicker();
    const fields: TaskEditFields = {
      title: this.input("text", "edit-title", task.title),
      project,
      dueDate: this.input("date", "edit-due", task.dueDate ?? ""),
      // The picker shows minutes; the core sends seconds too.
      remindAt: this.input("datetime-local", "edit-remind", task.remindAt?.slice(0, 16) ?? ""),
      repeat: this.select("edit-repeat", REPEATS.map((repeat): [string, string] => [repeat, this.translator.text(`repeat.${repeat}`)]), task.repeat),
      priority: this.select(
        "edit-priority",
        PRIORITIES.map((priority): [string, string] => [priority, this.translator.text(`priority.${priority}`)]),
        task.priority,
      ),
      status: this.select(
        "edit-status",
        TASK_STATUSES.map((status): [string, string] => [status, this.translator.text(`status.${status}`)]),
        task.status,
      ),
      waitingFor: this.input("text", "edit-waiting", task.waitingFor),
      kind: this.select(
        "edit-kind",
        ITEM_KINDS.map((kind): [string, string] => [kind, this.translator.text(`kind.${kind}`)]),
        task.kind,
      ),
      tags: this.input("text", "edit-tags", TagText.format(task.tags)),
      notes: ElementFactory.create("textarea", {
        className: "edit-notes",
        attributes: { id: "edit-notes", rows: "2", placeholder: this.translator.text("edit.notes_placeholder") },
      }),
    };
    fields.notes.value = task.notes;
    fields.waitingFor.placeholder = this.translator.text("edit.waiting_placeholder");

    const form = ElementFactory.create("form", { className: "edit-form" });
    const layout = () => form.replaceChildren(...this.layout(fields, picker.element));
    fields.kind.addEventListener("change", layout);
    fields.status.addEventListener("change", layout);
    layout();
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      this.save(task.taskId, fields);
    });
    form.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !picker.isOpen()) {
        this.handlers?.onCancel();
      }
    });
    queueMicrotask(() => fields.title.focus());
    return form;
  }

  /** The fields of the item's kind; "waits for" only while the task is waiting. */
  private layout(fields: TaskEditFields, projectPicker: HTMLElement): HTMLElement[] {
    const header = [
      this.field("edit.field.title", fields.title, "edit-field-wide"),
      this.field("edit.field.kind", fields.kind),
    ];
    if (fields.kind.value === "reminder") {
      return [
        ...header,
        this.field("capture.field.moment", fields.remindAt),
        this.field("capture.field.repeat", fields.repeat),
        this.field("edit.field.notes", fields.notes, "edit-field-wide"),
        this.actions(),
      ];
    }
    return [
      ...header,
      this.field("capture.field.project", projectPicker, "", "div"),
      this.field("capture.field.status", fields.status),
      ...(fields.status.value === "waiting" ? [this.field("edit.field.waiting", fields.waitingFor)] : []),
      this.field("capture.field.due", fields.dueDate),
      this.field("capture.field.priority", fields.priority),
      this.field("edit.field.remind", fields.remindAt),
      this.field("edit.field.tags", fields.tags),
      this.field("edit.field.notes", fields.notes, "edit-field-wide"),
      this.actions(),
    ];
  }

  private save(taskId: number, fields: TaskEditFields): void {
    if (fields.title.value.trim() === "") {
      fields.title.focus();
      return;
    }
    const reminder = fields.kind.value === "reminder";
    this.handlers?.onSave(taskId, {
      title: fields.title.value.trim(),
      notes: fields.notes.value.trim(),
      projectName: reminder ? null : fields.project.value,
      dueDate: reminder ? null : fields.dueDate.value || null,
      remindAt: fields.remindAt.value ? `${fields.remindAt.value}:00` : null,
      priority: reminder ? "normal" : (fields.priority.value as TaskEdit["priority"]),
      status: reminder ? "to_do" : (fields.status.value as TaskEdit["status"]),
      kind: fields.kind.value as TaskEdit["kind"],
      tags: reminder ? [] : TagText.parse(fields.tags.value),
      repeat: reminder ? (fields.repeat.value as TaskEdit["repeat"]) : "never",
      waitingFor: !reminder && fields.status.value === "waiting" ? fields.waitingFor.value.trim() : "",
    });
  }

  private actions(): HTMLElement {
    const cancel = ElementFactory.create("button", {
      className: "settings-button settings-button-quiet",
      text: this.translator.text("edit.cancel"),
      attributes: { type: "button" },
    });
    cancel.addEventListener("click", (event) => {
      event.stopPropagation();
      this.handlers?.onCancel();
    });
    const save = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("edit.save"),
      attributes: { type: "submit" },
    });
    save.addEventListener("click", (event) => event.stopPropagation());
    return ElementFactory.create("div", { className: "edit-actions edit-field-wide", children: [cancel, save] });
  }

  /**
   * A label and its control. The project picker goes in a plain block instead of a label:
   * a label would forward clicks inside the open list back to the picker's button.
   */
  private field(labelKey: string, control: HTMLElement, extraClass = "", tag: "label" | "div" = "label"): HTMLElement {
    return ElementFactory.create(tag, {
      className: `edit-field ${extraClass}`.trim(),
      children: [ElementFactory.create("span", { text: this.translator.text(labelKey) }), control],
    });
  }

  private input(type: string, id: string, value: string): HTMLInputElement {
    const input = ElementFactory.create("input", {
      className: "settings-input",
      attributes: { type, id, autocomplete: "off" },
    });
    input.value = value;
    return input;
  }

  private select(id: string, options: Array<[string, string]>, selected: string): HTMLSelectElement {
    const select = ElementFactory.create("select", {
      className: "settings-select",
      attributes: { id },
      children: options.map(([value, label]) => ElementFactory.create("option", { text: label, attributes: { value } })),
    });
    select.value = selected;
    return select;
  }
}
