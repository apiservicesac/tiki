import { PRIORITIES, TASK_STATUSES, type ClientSummary, type TaskDetails, type TaskEdit } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { ClientPicker } from "./ClientPicker";

export interface TaskEditHandlers {
  onSave: (taskId: number, edit: TaskEdit) => void;
  onCancel: () => void;
}

interface TaskEditFields {
  title: HTMLInputElement;
  /** The chosen client's name, or `null` for no client. */
  client: { value: string | null };
  dueDate: HTMLInputElement;
  remindAt: HTMLInputElement;
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

  render(task: TaskDetails, clients: readonly ClientSummary[], handlers: TaskEditHandlers): HTMLElement {
    this.handlers = handlers;
    if (this.shown?.taskId !== task.taskId) {
      this.shown = { taskId: task.taskId, element: this.build(task, clients) };
    }
    return this.shown.element;
  }

  /** Forgets the form, so the next task opens with fresh values. */
  reset(): void {
    this.shown = null;
  }

  private build(task: TaskDetails, clients: readonly ClientSummary[]): HTMLElement {
    const client = { value: task.clientName };
    const picker = new ClientPicker();
    const drawPicker = () => picker.update({
      id: "edit-client",
      options: [
        { value: null, label: this.translator.text("edit.no_client"), color: null },
        ...clients.map((summary) => ({ value: summary.name, label: summary.name, color: summary.color })),
      ],
      selected: client.value,
      searchPlaceholder: this.translator.text("tasks.search_client"),
      emptyText: this.translator.text("tasks.no_client_match"),
      onSelect: (value) => {
        client.value = value;
        drawPicker();
      },
    });
    drawPicker();
    const fields: TaskEditFields = {
      title: this.input("text", "edit-title", task.title),
      client,
      dueDate: this.input("date", "edit-due", task.dueDate ?? ""),
      // The picker shows minutes; the core sends seconds too.
      remindAt: this.input("datetime-local", "edit-remind", task.remindAt?.slice(0, 16) ?? ""),
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
        this.field("capture.field.client", picker.element, "", "div"),
        this.field("capture.field.due", fields.dueDate),
        this.field("edit.field.remind", fields.remindAt),
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
      clientName: fields.client.value,
      dueDate: fields.dueDate.value || null,
      remindAt: fields.remindAt.value ? `${fields.remindAt.value}:00` : null,
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

  /**
   * A label and its control. The client picker goes in a plain block instead of a label:
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
