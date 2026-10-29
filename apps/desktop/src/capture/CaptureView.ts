import type { InterpretedCapture, ProposedAction } from "../ai";
import { ProposedActionsView, type ApplyAction } from "../assistant/ProposedActionsView";
import { ITEM_KINDS, PRIORITIES, REPEATS, type ItemKind, type Priority, type ProjectSummary, type Repeat } from "../backend/contracts";
import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import { TagText } from "../dom/TagText";
import type { Translator } from "../i18n/Translator";
import { ProjectPicker } from "../island/ProjectPicker";

export interface CaptureViewHandlers {
  /** Enter in the box: the controller asks the AI, or saves the item. */
  onSubmit: () => void;
  onCancel: () => void;
  onOpenAiSettings: () => void;
}

/** What the box shows: nothing read yet, an item to review, or an answer. */
type BoxMode = "idle" | "capture" | "answer";

/**
 * Tiki's single box. The user writes anything: with AI, Enter reads it and either fills an
 * item's fields to review (a task or a reminder) or answers and proposes actions. Without
 * AI, the text is the title and the user picks the fields. Nothing is guessed with rules.
 *
 * A task shows project, due date, reminder, priority, tags and who it waits for; a reminder
 * shows only its moment and how often it repeats. The box is kept while it stays open, so
 * refreshes never wipe it.
 */
export class CaptureView {
  private element: HTMLElement | null = null;
  private sentence: HTMLInputElement | null = null;
  private kindToggle: HTMLElement | null = null;
  private fields: HTMLElement | null = null;
  private answer: HTMLElement | null = null;
  private status: HTMLElement | null = null;
  private dueDate: HTMLInputElement | null = null;
  private remindAt: HTMLInputElement | null = null;
  private priority: HTMLSelectElement | null = null;
  private repeat: HTMLSelectElement | null = null;
  private tags: HTMLInputElement | null = null;
  private waitingFor: HTMLInputElement | null = null;
  private readonly picker = new ProjectPicker();
  private readonly actionsView: ProposedActionsView;
  private project: string | null = null;
  private projects: readonly ProjectSummary[] = [];
  private kind: ItemKind = "task";
  private mode: BoxMode = "idle";
  private aiReady = false;
  /** What the AI understood and from which text, to learn from the user's corrections. */
  private reading: { sentence: string; understood: InterpretedCapture } | null = null;

  constructor(private readonly translator: Translator) {
    this.actionsView = new ProposedActionsView(translator);
  }

  /** The box, created on first use and reused until `reset`. */
  render(handlers: CaptureViewHandlers, projects: readonly ProjectSummary[], aiReady: boolean): HTMLElement {
    this.projects = projects;
    this.aiReady = aiReady;
    this.element ??= this.build(handlers);
    this.drawPicker();
    this.drawVisibility();
    if (this.mode === "idle") {
      this.setStatus(aiReady ? "capture.hint_ai" : "capture.hint_manual", !aiReady, handlers);
    }
    return this.element;
  }

  /** Forgets the box, so it opens empty next time. */
  reset(): void {
    this.element = null;
    this.project = null;
    this.kind = "task";
    this.mode = "idle";
    this.reading = null;
  }

  /** Whether Enter should ask the AI (nothing read yet, or after an answer) rather than save. */
  get needsReading(): boolean {
    return this.aiReady && this.mode !== "capture";
  }

  /** The text and what the AI understood from it, when the AI read an item. */
  get aiReading(): { sentence: string; understood: InterpretedCapture } | null {
    return this.reading;
  }

  get text(): string {
    return this.sentence?.value.trim() ?? "";
  }

  /** The item as the user sees it now: the text as title and the chosen fields. */
  values(): InterpretedCapture {
    const reminder = this.kind === "reminder";
    return {
      title: this.text,
      projectName: reminder ? null : this.project,
      dueDate: reminder ? null : this.dueDate?.value || null,
      priority: reminder ? "normal" : ((this.priority?.value as Priority | undefined) ?? "normal"),
      remindAt: CoreDateTimeText.fromPicker(this.remindAt?.value ?? ""),
      kind: this.kind,
      tags: reminder ? [] : TagText.parse(this.tags?.value ?? ""),
      repeat: reminder ? ((this.repeat?.value as Repeat | undefined) ?? "never") : "never",
      waitingFor: reminder ? null : this.waitingFor?.value.trim() || null,
    };
  }

  showThinking(): void {
    this.setStatus("capture.thinking", false);
    this.status?.classList.add("capture-status-thinking");
  }

  /** The AI's reading in the fields, for the user to review; the next Enter saves. */
  fill(item: InterpretedCapture): void {
    this.mode = "capture";
    this.reading = { sentence: this.text, understood: item };
    if (this.sentence) {
      this.sentence.value = item.title;
    }
    this.kind = item.kind;
    this.project = item.projectName;
    this.setValue(this.dueDate, item.dueDate ?? "");
    this.setValue(this.remindAt, CoreDateTimeText.toPicker(item.remindAt));
    this.setValue(this.priority, item.priority);
    this.setValue(this.repeat, item.repeat);
    this.setValue(this.tags, TagText.format(item.tags));
    this.setValue(this.waitingFor, item.waitingFor ?? "");
    this.drawKind();
    this.drawPicker();
    this.drawVisibility();
    this.setStatus("capture.review", false);
  }

  /** The AI answered a question, maybe proposing changes the user can apply. */
  showAnswer(text: string, actions: readonly ProposedAction[], apply: ApplyAction): void {
    this.mode = "answer";
    this.answer?.replaceChildren(
      ElementFactory.create("p", { className: "ask-answer", text }),
      ...this.actionsView.render(actions, apply),
    );
    this.drawVisibility();
    this.setStatus("capture.after_answer", false);
  }

  /** The AI could not read it: the user fills the fields, and the next Enter saves. */
  showAiFailed(): void {
    this.mode = "capture";
    this.drawVisibility();
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
    this.kindToggle = ElementFactory.create("div", { className: "tasks-toggle capture-kind", attributes: { role: "radiogroup" } });
    this.fields = ElementFactory.create("div", { className: "capture-fields" });
    this.answer = ElementFactory.create("div", { className: "capture-answer", attributes: { "aria-live": "polite" } });
    this.status = ElementFactory.create("p", { className: "island-hint capture-status" });
    this.buildFields();
    this.drawKind();

    const element = ElementFactory.create("div", {
      className: "capture",
      children: [this.sentence, this.kindToggle, this.fields, this.answer, this.status],
    });
    element.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && !this.picker.isOpen() && this.text !== "" && !(event.target instanceof HTMLTextAreaElement)) {
        event.preventDefault();
        handlers.onSubmit();
      } else if (event.key === "Escape" && !this.picker.isOpen()) {
        handlers.onCancel();
      }
    });
    queueMicrotask(() => this.sentence?.focus());
    return element;
  }

  private buildFields(): void {
    this.dueDate = this.input("date", "capture-due");
    this.remindAt = this.input("datetime-local", "capture-remind");
    this.priority = this.select("capture-priority", PRIORITIES.map((priority) => [priority, this.translator.text(`priority.${priority}`)]));
    this.priority.value = "normal";
    this.repeat = this.select("capture-repeat", REPEATS.map((repeat) => [repeat, this.translator.text(`repeat.${repeat}`)]));
    this.tags = this.input("text", "capture-tags", this.translator.text("edit.tags_placeholder"));
    this.waitingFor = this.input("text", "capture-waiting", this.translator.text("edit.waiting_placeholder"));
  }

  /** The fields of the current kind: a reminder only has its moment and its repeat. */
  private drawFields(): void {
    if (!this.fields || !this.dueDate || !this.remindAt || !this.priority || !this.repeat || !this.tags || !this.waitingFor) {
      return;
    }
    const children = this.kind === "reminder"
      ? [
        this.labelled("capture.field.moment", this.remindAt),
        this.labelled("capture.field.repeat", this.repeat),
      ]
      : [
        this.labelled("capture.field.project", this.picker.element, "div"),
        this.labelled("capture.field.due", this.dueDate),
        this.labelled("capture.field.remind", this.remindAt),
        this.labelled("capture.field.priority", this.priority),
        this.labelled("edit.field.waiting", this.waitingFor),
        this.labelled("edit.field.tags", this.tags),
      ];
    this.fields.replaceChildren(...children);
  }

  /** Fields show once there is an item to review, or always without AI; answers apart. */
  private drawVisibility(): void {
    const showFields = !this.aiReady || this.mode === "capture";
    if (this.fields) {
      this.fields.hidden = !showFields;
    }
    if (this.kindToggle) {
      this.kindToggle.hidden = !showFields;
    }
    if (this.answer) {
      this.answer.hidden = this.mode !== "answer";
    }
  }

  private drawKind(): void {
    const buttons = ITEM_KINDS.map((kind) => {
      const button = ElementFactory.textButton({
        className: kind === this.kind ? "tasks-toggle-option tasks-toggle-on" : "tasks-toggle-option",
        text: this.translator.text(`kind.${kind}`),
        attributes: { role: "radio", "aria-checked": String(kind === this.kind), "data-kind": kind },
      }, () => {
        this.kind = kind;
        this.drawKind();
      });
      return button;
    });
    this.kindToggle?.replaceChildren(...buttons);
    this.drawFields();
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
      createLabel: (name) => this.translator.format("projects.create", { name }),
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
      const link = ElementFactory.textButton({
        className: "capture-ai-link",
        text: this.translator.text("capture.set_up_ai"),
      }, handlers.onOpenAiSettings);
      children.push(document.createTextNode(UI_CONFIG.textMarks.separator), link);
    }
    this.status.replaceChildren(...children);
  }

  private setValue(control: HTMLInputElement | HTMLSelectElement | null, value: string): void {
    if (control) {
      control.value = value;
    }
  }

  private input(type: string, id: string, placeholder = ""): HTMLInputElement {
    return ElementFactory.create("input", {
      className: "settings-input",
      attributes: { type, id, autocomplete: "off", ...(placeholder ? { placeholder } : {}) },
    });
  }

  private select(id: string, options: Array<[string, string]>): HTMLSelectElement {
    return ElementFactory.create("select", {
      className: "settings-select",
      attributes: { id },
      children: options.map(([value, label]) => ElementFactory.create("option", { text: label, attributes: { value } })),
    });
  }

  private labelled(labelKey: string, control: HTMLElement, tag: "label" | "div" = "label"): HTMLElement {
    return ElementFactory.create(tag, {
      className: "edit-field",
      children: [ElementFactory.create("span", { text: this.translator.text(labelKey) }), control],
    });
  }
}
