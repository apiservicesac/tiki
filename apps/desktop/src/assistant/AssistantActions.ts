import type { IslandStore } from "../island/IslandStore";
import type { AskView } from "./AskView";
import type { TikiAssistant } from "./TikiAssistant";

/** "Ask Tiki" and the AI brief of the day. Without a key, both stay quiet. */
export class AssistantActions {
  constructor(
    private readonly store: IslandStore,
    private readonly assistant: TikiAssistant,
    private readonly askView: AskView,
  ) {}

  get ready(): boolean {
    return this.assistant.isReady(this.store.preferences());
  }

  async ask(question: string): Promise<void> {
    await this.showInAskPanel(() => this.assistant.answer(question, this.store.preferences()));
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
      this.askView.showAnswer(text);
      return text;
    } catch {
      this.askView.showFailure();
      return null;
    }
  }
}
