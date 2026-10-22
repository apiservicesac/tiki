import type { InterpretedCapture } from "../ai";
import { PRIORITIES, type Priority, type ProjectSummary } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { ProjectPicker } from "../island/ProjectPicker";

export interface CaptureViewHandlers {
  /** Enter in the sentence box: the controller decides whether the AI reads it or it is saved. */
  onSubmit: () => void;
  onCancel: () => void;
  onOpenAiSettings: () => void;
}

/**
 * Quick capture: a sentence box and the task's fields under it (project, due date,
 * reminder, priority). With AI, the first Enter fills the fields from the sentence and the
 * user reviews them; without AI, the user picks them. Nothing is guessed with word rules.
 *
 * The box is kept while the user stays in quick capture, so refreshes never wipe it.
 */
export class CaptureView {
  private element: HTMLElement | null = null;
  private sentence: HTMLInputElement | null = null;
  private dueDate: HTMLInputElement | null = null;
  private remindAt: HTMLInputElement | null = null;
  private priority: HTMLSelectElement | null = null;
  private status: HTMLElement | null = null;
  private readonly picker = new ProjectPicker();
  private project: string | null = null;
  private projects: readonly ProjectSummary[] = [];
  /** The AI already read this sentence: the next Enter saves. */
  private interpreted = false;

  constructor(private readonly translator: Translator) {}

  /** The capture box, created on first use and reused until `reset`. */
  render(handlers: CaptureViewHandlers, projects: readonly ProjectSummary[], aiReady: boolean): HTMLElement {
    this.projects = projects;
    this.element ??= this.build(handlers);
    this.drawPicker();
    if (!this.interpreted) {
      this.setStatus(aiReady ? "capture.hint_ai" : "capture.hint_manual", !aiReady, handlers);
    }
    return this.element;
  }

  /** Forgets the box, so the next capture starts empty. */
  reset(): void {
    this.element = null;
    this.project = null;
    this.interpreted = false;
  }

  /** Whether the AI has already filled the fields for the current sentence. */
  get isInterpreted(): boolean {
    return this.interpreted;
  }

  /** The task as the user sees it now: the sentence as title and the chosen fields. */
  values(): InterpretedCapture {
    return {
      title: this.sentence?.value.trim() ?? "",
      projectName: this.project,
      dueDate: this.dueDate?.value || null,
      priority: (this.priority?.value as Priority | undefined) ?? "normal",
      remindAt: this.remindAt?.value ? `${this.remindAt.value}:00` : null,
    };
  }

  /** The AI is reading the sentence. */
  showThinking(): void {
    this.setStatus("capture.thinking", false);
    this.status?.classList.add("capture-status-thinking");
  }

  /** The AI's reading, put in the fields for the user to review; the next Enter saves. */
  fill(capture: InterpretedCapture): void {
    this.interpreted = true;
    if (this.sentence) {
      this.sentence.value = capture.title;
    }
    this.project = capture.projectName;
    if (this.dueDate) {
      this.dueDate.value = capture.dueDate ?? "";
    }
    if (this.remindAt) {
      this.remindAt.value = capture.remindAt?.slice(0, 16) ?? "";
    }
    if (this.priority) {
      this.priority.value = capture.priority;
    }
    this.drawPicker();
    this.setStatus("capture.review", false);
  }

  /** The AI could not read it: the user fills the fields, and the next Enter saves. */
  showAiFailed(): void {
    this.interpreted = true;
    this.setStatus("capture.ai_failed", false);
  }

  private build(handlers: CaptureViewHandlers): HTMLElement {
    this.sentence = ElementFactory.create("input", {
      className: "capture-input",
      attributes: {
        type: "text",
        id: "capture-input",
        autocomplete: "off",
        spellcheck: "false",
        placeholder: this.translator.text("capture.placeholder"),
        "aria-label": this.translator.text("capture.placeholder"),
      },
    });
    this.dueDate = this.field("date", "capture-due");
    this.remindAt = this.field("datetime-local", "capture-remind");
    this.priority = ElementFactory.create("select", {
      className: "settings-select",
      attributes: { id: "capture-priority", "aria-label": this.translator.text("capture.field.priority") },
      children: PRIORITIES.map((priority) =>
        ElementFactory.create("option", { text: this.translator.text(`priority.${priority}`), attributes: { value: priority } })),
    });
    this.priority.value = "normal";
    this.status = ElementFactory.create("p", { className: "island-hint capture-status" });

    const element = ElementFactory.create("div", {
      className: "capture",
      children: [
        this.sentence,
        ElementFactory.create("div", {
          className: "capture-fields",
          children: [
            this.labelled("capture.field.project", this.picker.element, "div"),
            this.labelled("capture.field.due", this.dueDate),
            this.labelled("capture.field.remind", this.remindAt),
            this.labelled("capture.field.priority", this.priority),
          ],
        }),
        this.status,
      ],
    });
    element.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && !this.picker.isOpen() && (this.sentence?.value.trim() ?? "") !== "") {
        event.preventDefault();
        handlers.onSubmit();
      } else if (event.key === "Escape" && !this.picker.isOpen()) {
        handlers.onCancel();
      }
    });
    queueMicrotask(() => this.sentence?.focus());
    return element;
  }

  private drawPicker(): void {
    this.picker.update({
      id: "capture-project",
      options: [
        { value: null, label: this.translator.text("edit.no_project"), color: null },
        ...this.projects.map((project) => ({ value: project.name, label: project.name, color: project.color })),
      ],
      selected: this.project,
      searchPlaceholder: this.translator.text("tasks.search_project"),
      emptyText: this.translator.text("tasks.no_project_match"),
      onSelect: (value) => {
        this.project = value;
        this.drawPicker();
      },
    });
  }

  private setStatus(key: string, withAiLink: boolean, handlers?: CaptureViewHandlers): void {
    if (!this.status) {
      return;
    }
    this.status.className = "island-hint capture-status";
    const children: Node[] = [document.createTextNode(this.translator.text(key))];
    if (withAiLink && handlers) {
      const link = ElementFactory.create("button", {
        className: "capture-ai-link",
        text: this.translator.text("capture.set_up_ai"),
        attributes: { type: "button" },
      });
      link.addEventListener("click", (event) => {
        event.stopPropagation();
        handlers.onOpenAiSettings();
      });
      children.push(document.createTextNode(" · "), link);
    }
    this.status.replaceChildren(...children);
  }

  private field(type: string, id: string): HTMLInputElement {
    return ElementFactory.create("input", { className: "settings-input", attributes: { type, id, autocomplete: "off" } });
  }

  private labelled(labelKey: string, control: HTMLElement, tag: "label" | "div" = "label"): HTMLElement {
    return ElementFactory.create(tag, {
      className: "edit-field",
      children: [ElementFactory.create("span", { text: this.translator.text(labelKey) }), control],
    });
  }
}
