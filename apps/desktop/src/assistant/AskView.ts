import type { AskReply, ProposedAction } from "../ai";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface AskViewHandlers {
  onAsk: (question: string) => void;
  onSummarize: () => void;
  onOpenSettings: () => void;
  onCancel: () => void;
}

/** Applies one approved action; resolves `false` when it could not be applied. */
export type ApplyAction = (action: ProposedAction) => Promise<boolean>;

/**
 * "Ask Tiki": a question box, a button to summarize the day, the answer and the actions
 * Tiki proposes, each with its own Apply button. The box is kept while the panel stays
 * open, so refreshes never wipe the question or the answer.
 */
export class AskView {
  private element: HTMLElement | null = null;
  private answer: HTMLElement | null = null;
  private actions: HTMLElement | null = null;
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
    this.actions = null;
  }

  showThinking(): void {
    this.setAnswer(this.translator.text("capture.thinking"), "ask-answer ask-answer-thinking");
    this.actions?.replaceChildren();
  }

  /** The answer, and a card for each proposed action. */
  showReply(reply: AskReply, apply: ApplyAction): void {
    this.setAnswer(reply.text, "ask-answer");
    const cards = reply.actions.map((action) => this.actionCard(action, apply));
    const applyAll = reply.actions.length > 1 ? [this.applyAllButton(cards)] : [];
    this.actions?.replaceChildren(...cards, ...applyAll);
  }

  showFailure(): void {
    this.setAnswer(this.translator.text("ask.failed"), "ask-answer ask-answer-failed");
    this.actions?.replaceChildren();
  }

  private actionCard(action: ProposedAction, apply: ApplyAction): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-button ask-apply",
      text: this.translator.text("ask.apply"),
      attributes: { type: "button" },
    });
    const card = ElementFactory.create("div", {
      className: "ask-action",
      attributes: { "data-action": action.type },
      children: [ElementFactory.create("span", { className: "ask-action-summary", text: action.summary }), button],
    });
    button.addEventListener("click", async (event) => {
      event.stopPropagation();
      button.disabled = true;
      const applied = await apply(action);
      button.textContent = this.translator.text(applied ? "ask.applied" : "ask.apply_failed");
      card.classList.add(applied ? "ask-action-done" : "ask-action-failed");
      button.disabled = applied;
    });
    return card;
  }

  private applyAllButton(cards: HTMLElement[]): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "settings-button settings-button-quiet ask-apply-all",
      text: this.translator.text("ask.apply_all"),
      attributes: { type: "button" },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      button.disabled = true;
      cards.forEach((card) => card.querySelector<HTMLButtonElement>(".ask-apply:not(:disabled)")?.click());
    });
    return button;
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
    this.actions = ElementFactory.create("div", { className: "ask-actions" });
    queueMicrotask(() => input.focus());

    return ElementFactory.create("div", {
      className: "ask",
      children: [
        input,
        ElementFactory.create("div", { className: "settings-options", children: [summarize] }),
        this.answer,
        this.actions,
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
