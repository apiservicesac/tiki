import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface AskViewHandlers {
  onAsk: (question: string) => void;
  onSummarize: () => void;
  onOpenSettings: () => void;
  onCancel: () => void;
}

/**
 * "Ask Tiki": a question box, a button to summarize the day and the answer. The box is
 * kept while the panel stays open, so refreshes never wipe the question or the answer.
 */
export class AskView {
  private element: HTMLElement | null = null;
  private answer: HTMLElement | null = null;
  private handlers: AskViewHandlers | null = null;

  constructor(private readonly translator: Translator) {}

  render(ready: boolean, handlers: AskViewHandlers): HTMLElement {
    this.handlers = handlers;
    if (!ready) {
      return this.needsKey();
    }
    this.element ??= this.build();
    return this.element;
  }

  reset(): void {
    this.element = null;
    this.answer = null;
  }

  showThinking(): void {
    this.setAnswer(this.translator.text("capture.thinking"), "ask-answer ask-answer-thinking");
  }

  showAnswer(text: string): void {
    this.setAnswer(text, "ask-answer");
  }

  showFailure(): void {
    this.setAnswer(this.translator.text("ask.failed"), "ask-answer ask-answer-failed");
  }

  private setAnswer(text: string, className: string): void {
    if (this.answer) {
      this.answer.className = className;
      this.answer.textContent = text;
    }
  }

  private build(): HTMLElement {
    const input = ElementFactory.create("input", {
      className: "capture-input",
      attributes: {
        type: "text",
        id: "ask-input",
        autocomplete: "off",
        placeholder: this.translator.text("ask.placeholder"),
        "aria-label": this.translator.text("ask.placeholder"),
      },
    });
    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && input.value.trim() !== "") {
        this.handlers?.onAsk(input.value.trim());
      } else if (event.key === "Escape") {
        this.handlers?.onCancel();
      }
    });
    const summarize = ElementFactory.create("button", {
      className: "settings-option",
      text: this.translator.text("ask.summarize"),
      attributes: { type: "button", id: "ask-summarize" },
    });
    summarize.addEventListener("click", (event) => {
      event.stopPropagation();
      this.handlers?.onSummarize();
    });
    this.answer = ElementFactory.create("p", { className: "ask-answer", attributes: { "aria-live": "polite" } });
    queueMicrotask(() => input.focus());

    return ElementFactory.create("div", {
      className: "ask",
      children: [
        input,
        ElementFactory.create("div", { className: "settings-options", children: [summarize] }),
        this.answer,
        ElementFactory.create("p", { className: "island-hint", text: this.translator.text("ask.privacy") }),
      ],
    });
  }

  private needsKey(): HTMLElement {
    const open = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("ask.open_settings"),
      attributes: { type: "button" },
    });
    open.addEventListener("click", (event) => {
      event.stopPropagation();
      this.handlers?.onOpenSettings();
    });
    return ElementFactory.create("div", {
      className: "ask",
      children: [ElementFactory.create("p", { className: "island-empty", text: this.translator.text("ask.needs_key") }), open],
    });
  }
}
