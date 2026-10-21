import OpenAI from "openai";
import { zodTextFormat } from "openai/helpers/zod";
import { AI_CONFIG } from "./aiConfig";
import { AiError } from "./AiError";
import type { AiModelOption, ExtractionTask, StructuredModel } from "./contracts";

/** OpenAI models through the official OpenAI SDK and its Responses API. */
export class OpenAiModel implements StructuredModel {
  private readonly client: OpenAI;

  constructor(
    private readonly model: string,
    fetch: typeof globalThis.fetch,
  ) {
    this.client = new OpenAI({
      apiKey: AI_CONFIG.placeholderApiKey,
      fetch,
      // Runs inside Tiki's own window; the key stays in the shell, never in this page.
      dangerouslyAllowBrowser: true,
    });
  }

  async extract<Answer>(task: ExtractionTask<Answer>): Promise<Answer> {
    const response = await this.client.responses.parse({
      model: this.model,
      instructions: task.instructions,
      input: task.input,
      max_output_tokens: AI_CONFIG.providers.openai.maxTokens,
      text: { format: zodTextFormat(task.schema, task.schemaName) },
      ...(this.isReasoningModel() ? { reasoning: { effort: AI_CONFIG.extractionEffort } } : {}),
    });

    if (response.status === "incomplete") {
      throw new AiError("incomplete", "The model's answer was cut off.");
    }
    if (response.output_parsed === null) {
      throw new AiError("invalid_answer", "The model's answer did not match the expected shape.");
    }
    return response.output_parsed as Answer;
  }

  /** Chat models only, newest first. */
  async listModels(): Promise<AiModelOption[]> {
    const models: Array<{ id: string; created: number }> = [];
    for await (const model of this.client.models.list()) {
      if (OpenAiModel.isChatModel(model.id)) {
        models.push({ id: model.id, created: model.created });
      }
    }
    return models
      .sort((first, second) => second.created - first.created)
      .map(({ id }) => ({ id, label: id }));
  }

  private isReasoningModel(): boolean {
    return AI_CONFIG.openai.reasoningModelPrefixes.some((prefix) => this.model.startsWith(prefix));
  }

  private static isChatModel(id: string): boolean {
    return AI_CONFIG.openai.chatModelPrefixes.some((prefix) => id.startsWith(prefix))
      && !AI_CONFIG.openai.nonChatModelWords.some((word) => id.includes(word));
  }
}
