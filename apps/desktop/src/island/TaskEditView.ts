import { PRIORITIES, TASK_STATUSES, type TaskDetails, type TaskEdit } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface TaskEditHandlers {
  onSave: (taskId: number, edit: TaskEdit) => void;
  onCancel: () => void;
}

interface TaskEditFields {
  title: HTMLInputElement;
  client: HTMLSelectElement;
  dueDate: HTMLInputElement;
  priority: HTMLSelectElement;
  status: HTMLSelectElement;
  notes: HTMLTextAreaElement;
}

/**
 * The form to change a saved task. It is kept while the same task is open, so background
 * refreshes never wipe what the user is changing.
 */
export class TaskEditView {
  private shown: { taskId: number; element: HTMLElement } | null = null;
  private handlers: TaskEditHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(task: TaskDetails, clientNames: readonly string[], handlers: TaskEditHandlers): HTMLElement {
    this.handlers = handlers;
    if (this.shown?.taskId !== task.taskId) {
      this.shown = { taskId: task.taskId, element: this.build(task, clientNames) };
    }
    return this.shown.element;
  }

  /** Forgets the form, so the next task opens with fresh values. */
  reset(): void {
    this.shown = null;
  }

  private build(task: TaskDetails, clientNames: readonly string[]): HTMLElement {
    const fields: TaskEditFields = {
      title: this.input("text", "edit-title", task.title),
      client: this.select(
        "edit-client",
        [["", this.translator.text("edit.no_client")], ...clientNames.map((name): [string, string] => [name, name])],
        task.clientName ?? "",
      ),
      dueDate: this.input("date", "edit-due", task.dueDate ?? ""),
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
      notes: ElementFactory.create("textarea", {
        className: "edit-notes",
        attributes: { id: "edit-notes", rows: "2", placeholder: this.translator.text("edit.notes_placeholder") },
      }),
    };
    fields.notes.value = task.notes;

    const form = ElementFactory.create("form", {
      className: "edit-form",
      children: [
        this.field("edit.field.title", fields.title, "edit-field-wide"),
        this.field("capture.field.client", fields.client),
        this.field("capture.field.due", fields.dueDate),
        this.field("capture.field.priority", fields.priority),
        this.field("capture.field.status", fields.status),
        this.field("edit.field.notes", fields.notes, "edit-field-wide"),
        this.actions(),
      ],
    });
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      this.save(task.taskId, fields);
    });
    form.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
        this.handlers?.onCancel();
      }
    });
    queueMicrotask(() => fields.title.focus());
    return form;
  }

  private save(taskId: number, fields: TaskEditFields): void {
    if (fields.title.value.trim() === "") {
      fields.title.focus();
      return;
    }
    this.handlers?.onSave(taskId, {
      title: fields.title.value.trim(),
      notes: fields.notes.value.trim(),
      clientName: fields.client.value || null,
      dueDate: fields.dueDate.value || null,
      priority: fields.priority.value as TaskEdit["priority"],
      status: fields.status.value as TaskEdit["status"],
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

  private field(labelKey: string, control: HTMLElement, extraClass = ""): HTMLElement {
    return ElementFactory.create("label", {
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
