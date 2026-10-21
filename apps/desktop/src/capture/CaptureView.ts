import type { CaptureDraft, CaptureIntent } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface CaptureViewHandlers {
  onInput: (sentence: string) => void;
  onSubmit: (sentence: string) => void;
  onCancel: () => void;
}

/** The quick-capture box and the chips showing what Tiki understood. */
export class CaptureView {
  private chipsContainer: HTMLElement | null = null;

  constructor(private readonly translator: Translator) {}

  render(handlers: CaptureViewHandlers): HTMLElement {
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
    queueMicrotask(() => input.focus());

    return ElementFactory.create("div", {
      className: "capture",
      children: [
        input,
        this.chipsContainer,
        ElementFactory.create("p", {
          className: "island-hint",
          text: `${this.translator.text("capture.hint")} · ${this.translator.text("capture.client_tip")}`,
        }),
      ],
    });
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
