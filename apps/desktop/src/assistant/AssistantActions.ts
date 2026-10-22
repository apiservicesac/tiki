import type { ProposedAction } from "../ai";
import type { IslandStore } from "../island/IslandStore";
import type { ActionApplier } from "./ActionApplier";
import type { AskView } from "./AskView";
import type { TikiAssistant } from "./TikiAssistant";

/**
 * "Ask Tiki" and the AI brief of the day. The assistant may propose actions; each one
 * waits for the user's click. Without a key, both stay quiet.
 */
export class AssistantActions {
  constructor(
    private readonly store: IslandStore,
    private readonly assistant: TikiAssistant,
    private readonly askView: AskView,
    private readonly applier: ActionApplier,
  ) {}

  get ready(): boolean {
    return this.assistant.isReady(this.store.preferences());
  }

  async ask(question: string): Promise<void> {
    this.askView.showThinking();
    try {
      const reply = await this.assistant.ask(question, this.store.preferences());
      this.askView.showReply(reply, (action) => this.apply(action));
    } catch {
      this.askView.showFailure();
    }
  }

  /** Applies one approved action and refreshes the island; `false` when it failed. */
  async apply(action: ProposedAction): Promise<boolean> {
    try {
      await this.applier.apply(action);
      await this.store.refresh();
      return true;
    } catch {
      return false;
    }
  }

  /** Summarizes the day in the Ask panel and keeps it on the Today tab. */
  async summarize(): Promise<void> {
    const brief = await this.showInAskPanel(() => this.assistant.summarizeDay(this.store.preferences()));
    if (brief) {
      this.store.update({ dayBrief: brief });
    }
  }

  /** Prepares the AI brief in the background, for the Today tab. Failures are ignored. */
  async prepareDayBrief(): Promise<void> {
    if (!this.ready) {
      return;
    }
    try {
      this.store.update({ dayBrief: await this.assistant.summarizeDay(this.store.preferences()) });
    } catch {
      // The rules-based list is still there; the brief is a bonus.
    }
  }

  private async showInAskPanel(request: () => Promise<string>): Promise<string | null> {
    this.askView.showThinking();
    try {
      const text = await request();
      this.askView.showReply({ text, actions: [] }, (action) => this.apply(action));
      return text;
    } catch {
      this.askView.showFailure();
      return null;
    }
  }
}
