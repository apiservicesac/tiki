import type { ProposedAction } from "../ai";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

/** Applies one approved action; resolves `false` when it could not be applied. */
export type ApplyAction = (action: ProposedAction) => Promise<boolean>;

/** The changes Tiki proposes, each with its own Apply button, and "Apply all" for several. */
export class ProposedActionsView {
  constructor(private readonly translator: Translator) {}

  render(actions: readonly ProposedAction[], apply: ApplyAction): HTMLElement[] {
    const cards = actions.map((action) => this.card(action, apply));
    return actions.length > 1 ? [...cards, this.applyAllButton(cards)] : cards;
  }

  private card(action: ProposedAction, apply: ApplyAction): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-button ask-apply",
      text: this.translator.text("ask.apply"),
    }, async () => {
      button.disabled = true;
      const applied = await apply(action);
      button.textContent = this.translator.text(applied ? "ask.applied" : "ask.apply_failed");
      card.classList.add(applied ? "ask-action-done" : "ask-action-failed");
      button.disabled = applied;
    });
    const card = ElementFactory.create("div", {
      className: "ask-action",
      attributes: { "data-action": action.type },
      children: [ElementFactory.create("span", { className: "ask-action-summary", text: action.summary }), button],
    });
    return card;
  }

  private applyAllButton(cards: HTMLElement[]): HTMLElement {
    const button = ElementFactory.textButton({
      className: "settings-button settings-button-quiet ask-apply-all",
      text: this.translator.text("ask.apply_all"),
    }, () => {
      button.disabled = true;
      cards.forEach((card) => card.querySelector<HTMLButtonElement>(".ask-apply:not(:disabled)")?.click());
    });
    return button;
  }
}
