import { AI_CONFIG } from "./aiConfig";
import { AnthropicModel } from "./AnthropicModel";
import type { AiModelOption, AiProviderName, SendThroughShell, StructuredModel } from "./contracts";
import { OpenAiModel } from "./OpenAiModel";
import { ShellFetch } from "./ShellFetch";

/**
 * Entry point of Tiki's AI library: picks the provider's official SDK and keeps every
 * request going through the desktop shell, which holds the keys.
 */
export class TikiAi {
  private readonly shellFetch: typeof globalThis.fetch;

  constructor(sendThroughShell: SendThroughShell) {
    this.shellFetch = new ShellFetch(sendThroughShell).fetch;
  }

  model(provider: AiProviderName, modelId: string | null): StructuredModel {
    return this.client(provider, modelId ?? AI_CONFIG.providers[provider].defaultModel);
  }

  /**
   * The provider's live model list when a key is configured; the curated fallback list
   * when it is not, or when the provider cannot be reached.
   */
  async availableModels(provider: AiProviderName, keyConfigured: boolean): Promise<AiModelOption[]> {
    const fallback = [...AI_CONFIG.providers[provider].fallbackModels];
    if (!keyConfigured) {
      return fallback;
    }
    try {
      const live = await this.client(provider, AI_CONFIG.providers[provider].defaultModel).listModels();
      return live.length > 0 ? live : fallback;
    } catch {
      return fallback;
    }
  }

  private client(provider: AiProviderName, modelId: string): AnthropicModel | OpenAiModel {
    return provider === "anthropic"
      ? new AnthropicModel(modelId, this.shellFetch)
      : new OpenAiModel(modelId, this.shellFetch);
  }
}
