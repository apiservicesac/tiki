import type { InterpretedCapture } from "../ai";
import type { CaptureDraft, CaptureIntent } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface CaptureViewHandlers {
  onInput: (sentence: string) => void;
  onSubmit: (sentence: string) => void;
  onCancel: () => void;
}

/**
 * The quick-capture box and the chips showing what Tiki understood. The box is kept while
 * the user stays in quick capture, so refreshes never wipe what they are typing.
 */
export class CaptureView {
  private chipsContainer: HTMLElement | null = null;
  private hint: HTMLElement | null = null;
  private element: HTMLElement | null = null;

  constructor(private readonly translator: Translator) {}

  /** The capture box, created on first use and reused until `reset`. */
  render(handlers: CaptureViewHandlers): HTMLElement {
    this.element ??= this.build(handlers);
    return this.element;
  }

  /** Forgets the box, so the next capture starts empty. */
  reset(): void {
    this.element = null;
    this.chipsContainer = null;
    this.hint = null;
  }

  private build(handlers: CaptureViewHandlers): HTMLElement {
    const input = ElementFactory.create("input", {
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
    input.addEventListener("input", () => handlers.onInput(input.value));
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && input.value.trim() !== "") {
        handlers.onSubmit(input.value);
      } else if (event.key === "Escape") {
        handlers.onCancel();
      }
    });

    this.chipsContainer = ElementFactory.create("div", { className: "capture-chips" });
    this.hint = ElementFactory.create("p", { className: "island-hint", text: this.defaultHint() });
    queueMicrotask(() => input.focus());

    return ElementFactory.create("div", {
      className: "capture",
      children: [input, this.chipsContainer, this.hint],
    });
  }

  /** The assistant is reading the sentence. */
  showThinking(): void {
    this.chipsContainer?.replaceChildren(
      ElementFactory.create("span", { className: "capture-thinking", text: this.translator.text("capture.thinking") }),
    );
  }

  /** What the assistant understood; a second Enter saves it. */
  showInterpreted(capture: InterpretedCapture): void {
    const chips = [this.chip("capture.field.title", capture.title)];
    if (capture.clientName) {
      chips.push(this.chip("capture.field.client", capture.clientName));
    }
    if (capture.dueDate) {
      chips.push(this.chip("capture.field.due", this.formatDate(capture.dueDate)));
    }
    chips.push(this.chip("capture.field.priority", this.translator.text(`priority.${capture.priority}`)));
    this.chipsContainer?.replaceChildren(...chips);
    this.setHint(`${this.translator.text("capture.confirm")} · ${this.translator.text("capture.edit_tip")}`);
  }

  /** Replaces the chips with what the core understood from the current sentence. */
  showIntent(intent: CaptureIntent | null): void {
    if (!this.chipsContainer) {
      return;
    }
    const chips = !intent
      ? []
      : intent.kind === "client"
        ? [this.chip("capture.new_client", intent.name)]
        : this.taskChips(intent);
    this.chipsContainer.replaceChildren(...chips);
    this.setHint(this.defaultHint());
  }

  private defaultHint(): string {
    return `${this.translator.text("capture.hint")} · ${this.translator.text("capture.client_tip")}`;
  }

  private setHint(text: string): void {
    if (this.hint) {
      this.hint.textContent = text;
    }
  }

  private taskChips(draft: CaptureDraft): HTMLElement[] {
    const chips: HTMLElement[] = [];
    if (draft.clientName) {
      chips.push(this.chip("capture.field.client", draft.clientName));
    }
    if (draft.dueDate) {
      chips.push(this.chip("capture.field.due", this.formatDate(draft.dueDate)));
    }
    chips.push(this.chip("capture.field.status", this.translator.text("status.to_do")));
    return chips;
  }

  private chip(labelKey: string, value: string): HTMLElement {
    return ElementFactory.create("span", {
      className: "capture-chip",
      children: [
        ElementFactory.create("em", { text: this.translator.text(labelKey) }),
        document.createTextNode(` ${value}`),
      ],
    });
  }

  private formatDate(isoDate: string): string {
    const [year, month, day] = isoDate.split("-").map(Number) as [number, number, number];
    return new Intl.DateTimeFormat(this.translator.localeCode, {
      weekday: "short",
      day: "numeric",
      month: "short",
    }).format(new Date(year, month - 1, day));
  }
}
