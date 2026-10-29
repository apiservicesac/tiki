import {
  ITEM_KINDS,
  PRIORITIES,
  REPEATS,
  TASK_STATUSES,
  type ItemKind,
  type Priority,
  type ProjectSummary,
  type Repeat,
  type TaskStatus,
} from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { ProjectPicker } from "./ProjectPicker";

export interface InputOptions {
  value?: string;
  placeholder?: string;
}

export interface FieldLayout {
  /** Spans both columns of the form. */
  wide?: boolean;
  /**
   * A plain block instead of a label, for the project picker: a label would forward clicks
   * inside the open list back to the picker's button.
   */
  block?: boolean;
}

/** The project the user picked, shared with the picker's callbacks. */
export interface ProjectChoice {
  value: string | null;
}

/**
 * The fields of an item, built the same way in quick capture and in the edit form: inputs,
 * the priority, repeat and status lists, the task-or-reminder toggle, the reminder's moment,
 * the project picker, and each control under its label.
 */
export class ItemFieldsBuilder {
  constructor(private readonly translator: Translator) {}

  input(type: string, id: string, options: InputOptions = {}): HTMLInputElement {
    const input = ElementFactory.create("input", {
      className: "settings-input",
      attributes: { type, id, autocomplete: "off", ...(options.placeholder ? { placeholder: options.placeholder } : {}) },
    });
    input.value = options.value ?? "";
    return input;
  }

  /** The moment a reminder rings, in a date-and-time picker. */
  remindAt(id: string, moment: string | null): HTMLInputElement {
    return this.input("datetime-local", id, { value: CoreDateTimeText.toPicker(moment) });
  }

  priority(id: string, selected: Priority): HTMLSelectElement {
    return this.select(id, PRIORITIES, "priority", selected);
  }

  repeat(id: string, selected: Repeat): HTMLSelectElement {
    return this.select(id, REPEATS, "repeat", selected);
  }

  /** Task or reminder, as two buttons side by side; `onSelect` receives the one chosen. */
  kindToggle(selected: ItemKind, onSelect: (kind: ItemKind) => void): HTMLElement {
    const options = ITEM_KINDS.map((kind) => ElementFactory.textButton({
      className: kind === selected ? "tasks-toggle-option tasks-toggle-on" : "tasks-toggle-option",
      text: this.translator.text(`kind.${kind}`),
      attributes: { role: "radio", "aria-checked": String(kind === selected), "data-kind": kind },
    }, () => onSelect(kind)));
    return ElementFactory.create("div", { className: "tasks-toggle item-kind", attributes: { role: "radiogroup" }, children: options });
  }

  status(id: string, selected: TaskStatus): HTMLSelectElement {
    return this.select(id, TASK_STATUSES, "status", selected);
  }

  /** Shows the projects in the picker, with "No project" first; a new name can be typed. */
  drawProjectPicker(picker: ProjectPicker, id: string, projects: readonly ProjectSummary[], choice: ProjectChoice): void {
    picker.update({
      id,
      options: [
        { value: null, label: this.translator.text("edit.no_project"), color: null },
        ...projects.map((project) => ({ value: project.name, label: project.name, color: project.color })),
      ],
      selected: choice.value,
      searchPlaceholder: this.translator.text("tasks.search_project"),
      emptyText: this.translator.text("tasks.no_project_match"),
      createLabel: (name) => this.translator.format("projects.create", { name }),
      onSelect: (value) => {
        choice.value = value;
        this.drawProjectPicker(picker, id, projects, choice);
      },
    });
  }

  /** A control under its label. */
  field(labelKey: string, control: HTMLElement, layout: FieldLayout = {}): HTMLElement {
    return ElementFactory.create(layout.block ? "div" : "label", {
      className: layout.wide ? "edit-field edit-field-wide" : "edit-field",
      children: [ElementFactory.create("span", { text: this.translator.text(labelKey) }), control],
    });
  }

  /** A list of values, each labelled by its translation (`priority.high`). */
  private select(id: string, values: readonly string[], keyPrefix: string, selected: string): HTMLSelectElement {
    const select = ElementFactory.create("select", {
      className: "settings-select",
      attributes: { id },
      children: values.map((value) =>
        ElementFactory.create("option", { text: this.translator.text(`${keyPrefix}.${value}`), attributes: { value } })),
    });
    select.value = selected;
    return select;
  }
}
