import { BackendError } from "../backend/BackendError";
import type { ProjectSummary, TaskDetails, TaskEdit } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import { TagText } from "../dom/TagText";
import type { Translator } from "../i18n/Translator";
import { ItemFieldsBuilder, type ProjectChoice } from "./ItemFieldsBuilder";
import { ProjectPicker } from "./ProjectPicker";

export interface TaskEditHandlers {
  /** Rejects when the core refuses the change; the form then says what to fix. */
  onSave: (taskId: number, edit: TaskEdit) => Promise<void>;
  onCancel: () => void;
}

interface TaskEditFields {
  title: HTMLInputElement;
  /** The chosen project's name, or `null` for no project; a new name is created on save. */
  project: ProjectChoice;
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
  private readonly fieldsBuilder: ItemFieldsBuilder;

  constructor(private readonly translator: Translator) {
    this.fieldsBuilder = new ItemFieldsBuilder(translator);
  }

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
    const project: ProjectChoice = { value: task.projectName };
    const picker = new ProjectPicker();
    this.fieldsBuilder.drawProjectPicker(picker, "edit-project", projects, project);
    const fields: TaskEditFields = {
      title: this.fieldsBuilder.input("text", "edit-title", { value: task.title }),
      project,
      dueDate: this.fieldsBuilder.input("date", "edit-due", { value: task.dueDate ?? "" }),
      remindAt: this.fieldsBuilder.remindAt("edit-remind", task.remindAt),
      repeat: this.fieldsBuilder.repeat("edit-repeat", task.repeat),
      priority: this.fieldsBuilder.priority("edit-priority", task.priority),
      status: this.fieldsBuilder.status("edit-status", task.status),
      waitingFor: this.fieldsBuilder.input("text", "edit-waiting", {
        value: task.waitingFor,
        placeholder: this.translator.text("edit.waiting_placeholder"),
      }),
      kind: this.fieldsBuilder.kind("edit-kind", task.kind),
      tags: this.fieldsBuilder.input("text", "edit-tags", { value: TagText.format(task.tags) }),
      notes: ElementFactory.create("textarea", {
        className: "edit-notes",
        attributes: { id: "edit-notes", rows: "2", placeholder: this.translator.text("edit.notes_placeholder") },
      }),
    };
    fields.notes.value = task.notes;
    const problem = ElementFactory.create("p", { className: "island-hint edit-problem edit-field-wide", attributes: { role: "alert" } });
    problem.hidden = true;

    const form = ElementFactory.create("form", { className: "edit-form" });
    const layout = () => form.replaceChildren(...this.layout(fields, picker.element), problem, this.actions());
    fields.kind.addEventListener("change", layout);
    fields.status.addEventListener("change", layout);
    layout();
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      this.save(task.taskId, fields, problem);
    });
    form.addEventListener("keydown", (event) => {
      if (event.key === "Escape" && !picker.isOpen()) {
        this.handlers?.onCancel();
      }
    });
    queueMicrotask(() => fields.title.focus());
    return form;
  }

  /** The fields of the item's kind; "waits for" only while the task is waiting. The problem line and the buttons follow. */
  private layout(fields: TaskEditFields, projectPicker: HTMLElement): HTMLElement[] {
    const header = [
      this.fieldsBuilder.field("edit.field.title", fields.title, { wide: true }),
      this.fieldsBuilder.field("edit.field.kind", fields.kind),
    ];
    if (fields.kind.value === "reminder") {
      return [
        ...header,
        this.fieldsBuilder.field("capture.field.moment", fields.remindAt),
        this.fieldsBuilder.field("capture.field.repeat", fields.repeat),
        this.fieldsBuilder.field("edit.field.notes", fields.notes, { wide: true }),
      ];
    }
    return [
      ...header,
      this.fieldsBuilder.field("capture.field.project", projectPicker, { block: true }),
      this.fieldsBuilder.field("capture.field.status", fields.status),
      ...(fields.status.value === "waiting" ? [this.fieldsBuilder.field("edit.field.waiting", fields.waitingFor)] : []),
      this.fieldsBuilder.field("capture.field.due", fields.dueDate),
      this.fieldsBuilder.field("capture.field.priority", fields.priority),
      this.fieldsBuilder.field("edit.field.remind", fields.remindAt),
      this.fieldsBuilder.field("edit.field.tags", fields.tags),
      this.fieldsBuilder.field("edit.field.notes", fields.notes, { wide: true }),
    ];
  }

  /** Sends the changes; if the core refuses them, the form stays filled and says what to fix. */
  private save(taskId: number, fields: TaskEditFields, problem: HTMLElement): void {
    if (fields.title.value.trim() === "") {
      fields.title.focus();
      return;
    }
    const reminder = fields.kind.value === "reminder";
    problem.hidden = true;
    this.handlers?.onSave(taskId, {
      title: fields.title.value.trim(),
      notes: fields.notes.value.trim(),
      projectName: reminder ? null : fields.project.value,
      dueDate: reminder ? null : fields.dueDate.value || null,
      remindAt: CoreDateTimeText.fromPicker(fields.remindAt.value),
      priority: reminder ? "normal" : (fields.priority.value as TaskEdit["priority"]),
      status: reminder ? "to_do" : (fields.status.value as TaskEdit["status"]),
      kind: fields.kind.value as TaskEdit["kind"],
      tags: reminder ? [] : TagText.parse(fields.tags.value),
      repeat: reminder ? (fields.repeat.value as TaskEdit["repeat"]) : "never",
      waitingFor: !reminder && fields.status.value === "waiting" ? fields.waitingFor.value.trim() : "",
    }).catch((error: unknown) => {
      problem.textContent = this.translator.text(BackendError.messageKeyOf(error));
      problem.hidden = false;
    });
  }

  private actions(): HTMLElement {
    const cancel = ElementFactory.textButton({
      className: "settings-button settings-button-quiet",
      text: this.translator.text("edit.cancel"),
    }, () => this.handlers?.onCancel());
    const save = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("edit.save"),
      attributes: { type: "submit" },
    });
    save.addEventListener("click", (event) => event.stopPropagation());
    return ElementFactory.create("div", { className: "edit-actions edit-field-wide", children: [cancel, save] });
  }
}
