import Anthropic from "@anthropic-ai/sdk";
import { betaZodOutputFormat } from "@anthropic-ai/sdk/helpers/beta/zod";
import { AI_CONFIG } from "./aiConfig";
import { AiError } from "./AiError";
import type { AiModelOption, ExtractionTask, StructuredModel } from "./contracts";

/** Claude through the official Anthropic SDK. */
export class AnthropicModel implements StructuredModel {
  private readonly client: Anthropic;

  constructor(
    private readonly model: string,
    fetch: typeof globalThis.fetch,
  ) {
    this.client = new Anthropic({
      apiKey: AI_CONFIG.placeholderApiKey,
      fetch,
      // Runs inside Tiki's own window; the key stays in the shell, never in this page.
      dangerouslyAllowBrowser: true,
    });
  }

  async extract<Answer>(task: ExtractionTask<Answer>): Promise<Answer> {
    const response = await this.client.beta.messages.parse({
      model: this.model,
      max_tokens: AI_CONFIG.providers.anthropic.maxTokens,
      system: task.instructions,
      messages: [{ role: "user", content: task.input }],
      output_config: {
        format: betaZodOutputFormat(task.schema),
        ...(this.supportsEffort() ? { effort: AI_CONFIG.extractionEffort } : {}),
      },
      ...(this.supportsServerFallback() ? { betas: [AI_CONFIG.anthropic.fallbackBeta], fallbacks: "default" as const } : {}),
    });

    if (response.stop_reason === "refusal") {
      throw new AiError("refused", "Claude declined this request.");
    }
    if (response.stop_reason === "max_tokens") {
      throw new AiError("incomplete", "Claude's answer was cut off.");
    }
    if (response.parsed_output === null) {
      throw new AiError("invalid_answer", "Claude's answer did not match the expected shape.");
    }
    return response.parsed_output as Answer;
  }

  async listModels(): Promise<AiModelOption[]> {
    const models: AiModelOption[] = [];
    for await (const model of this.client.models.list()) {
      models.push({ id: model.id, label: model.display_name });
    }
    return models;
  }

  private supportsEffort(): boolean {
    return !AI_CONFIG.anthropic.modelsWithoutEffort.some((prefix) => this.model.startsWith(prefix));
  }

  private supportsServerFallback(): boolean {
    return AI_CONFIG.anthropic.modelsWithServerFallback.includes(this.model);
  }
}
