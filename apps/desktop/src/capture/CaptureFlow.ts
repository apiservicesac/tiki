import type { AssistantActions } from "../assistant/AssistantActions";
import type { TikiAssistant } from "../assistant/TikiAssistant";
import type { TikiBackend } from "../backend/contracts";
import type { IslandStore } from "../island/IslandStore";
import type { CaptureView } from "./CaptureView";

/**
 * Enter in the single box. With an AI key, Tiki reads the text: an item fills its fields
 * for the user to review (the next Enter saves), a question gets an answer and maybe
 * actions to apply. Without a key, the text is the title and Enter saves.
 */
export class CaptureFlow {
  constructor(
    private readonly view: CaptureView,
    private readonly backend: TikiBackend,
    private readonly assistant: TikiAssistant,
    private readonly assistantActions: AssistantActions,
    private readonly store: IslandStore,
  ) {}

  async submit(): Promise<void> {
    if (this.view.needsReading) {
      await this.read();
    } else {
      await this.save();
    }
  }

  private async read(): Promise<void> {
    const text = this.view.text;
    this.view.showThinking();
    try {
      const reply = await this.assistant.command(text, this.store.preferences());
      if (reply.kind === "capture") {
        this.view.fill(reply.item);
      } else {
        this.view.showAnswer(reply.text, reply.actions, (action) => this.assistantActions.apply(action));
      }
    } catch {
      this.view.showAiFailed();
    }
  }

  private async save(): Promise<void> {
    const capture = this.view.values();
    if (capture.title === "") {
      return;
    }
    await this.backend.saveConfirmedCapture(capture);
    const reading = this.view.aiReading;
    if (reading) {
      // What the user changed before saving teaches the AI their way of saying things.
      void this.assistant.learn(reading.sentence, reading.understood, capture).catch(() => undefined);
    }
    this.view.reset();
    await this.store.refresh();
    this.store.update({ mode: "expanded", panel: "tasks" });
    this.store.react("happy");
  }
}
