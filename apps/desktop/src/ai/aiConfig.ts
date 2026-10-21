import type { AiModelOption, AiProviderName } from "./contracts";

interface ProviderConfig {
  /** Used when the user has not picked a model. */
  defaultModel: string;
  /** Shown when the live list cannot be read (no key yet, offline): fast, balanced and strongest. */
  fallbackModels: readonly AiModelOption[];
  /** Upper bound for one structured answer. */
  maxTokens: number;
}

/** Every tunable value of the AI library lives here, nowhere else. */
export const AI_CONFIG = Object.freeze({
  /**
   * The SDKs require a key to build requests; the real one is added by the desktop shell
   * from the operating system's credential store and never reaches the interface.
   */
  placeholderApiKey: "added-by-tiki-shell",
  /** Short extractions (a task from a sentence) need little reasoning. */
  extractionEffort: "low" as const,
  providers: Object.freeze<Record<AiProviderName, ProviderConfig>>({
    anthropic: {
      defaultModel: "claude-opus-5-5",
      maxTokens: 16_000,
      fallbackModels: [
        { id: "claude-opus-5-5", label: "Claude Opus 5.5" },
        { id: "claude-sonnet-5-5", label: "Claude Sonnet 5.5" },
        { id: "claude-haiku-5-5", label: "Claude Haiku 5.5" },
        { id: "claude-fable-5-1", label: "Claude Fable 5.1" },
        { id: "claude-opus-4-8", label: "Claude Opus 4.8" },
      ],
    },
    openai: {
      defaultModel: "gpt-5.5",
      maxTokens: 16_000,
      fallbackModels: [
        { id: "gpt-5.5", label: "GPT-5.5" },
        { id: "gpt-6.1-sol", label: "GPT-6.1 Sol" },
        { id: "gpt-5.4", label: "GPT-5.4" },
        { id: "gpt-5.4-mini", label: "GPT-5.4 mini" },
        { id: "gpt-5.4-nano", label: "GPT-5.4 nano" },
      ],
    },
  }),
  anthropic: Object.freeze({
    /** Server-side refusal fallback, opted in for the models that accept the "default" form. */
    fallbackBeta: "server-side-fallback-2026-07-01",
    modelsWithServerFallback: Object.freeze(["claude-opus-5-5", "claude-opus-5", "claude-sonnet-5-5", "claude-fable-5-1"]),
    /** Older models reject the `effort` setting. */
    modelsWithoutEffort: Object.freeze(["claude-haiku-4", "claude-sonnet-4-5", "claude-opus-4-1", "claude-3"]),
  }),
  openai: Object.freeze({
    /** The live list also has audio, image and embedding models; only chat models are offered. */
    chatModelPrefixes: Object.freeze(["gpt-", "o"]),
    nonChatModelWords: Object.freeze(["audio", "realtime", "tts", "transcribe", "image", "search", "embedding", "moderation", "codex", "computer", "deep-research", "instruct"]),
    /** Only reasoning models accept a reasoning effort. */
    reasoningModelPrefixes: Object.freeze(["gpt-5", "gpt-6", "o"]),
  }),
});
