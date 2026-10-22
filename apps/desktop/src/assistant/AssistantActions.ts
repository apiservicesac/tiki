import type { ProposedAction } from "../ai";
import type { IslandStore } from "../island/IslandStore";
import type { ActionApplier } from "./ActionApplier";
import type { TikiAssistant } from "./TikiAssistant";

/**
 * The AI brief of the day and the actions Tiki proposes: each action waits for the user's
 * click. Without a key, both stay quiet.
 */
export class AssistantActions {
  constructor(
    private readonly store: IslandStore,
    private readonly assistant: TikiAssistant,
    private readonly applier: ActionApplier,
  ) {}

  get ready(): boolean {
    return this.assistant.isReady(this.store.preferences());
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

  /** Writes the AI brief of the day for the summary card on Today. Failures are ignored. */
  async prepareDayBrief(): Promise<void> {
    if (!this.ready) {
      return;
    }
    this.store.update({ preparingBrief: true });
    try {
      this.store.update({ dayBrief: await this.assistant.summarizeDay(this.store.preferences()) });
    } catch {
      // The list of the day is still there; the brief is a bonus.
    } finally {
      this.store.update({ preparingBrief: false });
    }
  }
}
