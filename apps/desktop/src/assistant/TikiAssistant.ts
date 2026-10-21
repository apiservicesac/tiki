import {
  CaptureInterpreter,
  DayAdvisor,
  TikiAi,
  type AdvisorLanguage,
  type AiModelOption,
  type AiProviderName,
  type InterpretedCapture,
} from "../ai";
import { AI_PROVIDERS, type LocaleName, type TikiBackend, type UserPreferences } from "../backend/contracts";

export type ConfiguredProviders = Record<AiProviderName, boolean>;

const ADVISOR_LANGUAGES: Readonly<Record<LocaleName, AdvisorLanguage>> = Object.freeze({
  spanish: "Spanish",
  english: "English",
});

/** Connects the AI library to the user's data: keys status, model choice and quick capture. */
export class TikiAssistant {
  private configured: ConfiguredProviders = TikiAssistant.noProviders();

  constructor(
    private readonly backend: TikiBackend,
    private readonly ai: TikiAi,
  ) {}

  /** Asks the shell which providers have a key stored. */
  async loadStatus(): Promise<ConfiguredProviders> {
    const statuses = await this.backend.aiStatus();
    const configured = TikiAssistant.noProviders();
    statuses.forEach((status) => {
      configured[status.provider] = status.configured;
    });
    this.configured = configured;
    return { ...configured };
  }

  isReady(preferences: UserPreferences): boolean {
    return this.configured[preferences.aiProvider];
  }

  models(provider: AiProviderName): Promise<AiModelOption[]> {
    return this.ai.availableModels(provider, this.configured[provider]);
  }

  /** Reads a sentence with the chosen model; only the sentence and client names are sent. */
  async interpret(sentence: string, preferences: UserPreferences): Promise<InterpretedCapture> {
    const clientNames = await this.backend.clientNames();
    const interpreter = new CaptureInterpreter(this.ai.model(preferences.aiProvider, preferences.aiModel));
    return interpreter.interpret(sentence, { clientNames, today: new Date() });
  }

  /** A short AI brief of the user's open tasks. */
  async summarizeDay(preferences: UserPreferences): Promise<string> {
    const context = await this.backend.assistantContext();
    return this.advisor(preferences).summarizeDay(context, ADVISOR_LANGUAGES[preferences.locale]);
  }

  /** Answers a question about the user's open tasks. */
  async answer(question: string, preferences: UserPreferences): Promise<string> {
    const context = await this.backend.assistantContext();
    return this.advisor(preferences).answer(question, context, ADVISOR_LANGUAGES[preferences.locale]);
  }

  private advisor(preferences: UserPreferences): DayAdvisor {
    return new DayAdvisor(this.ai.model(preferences.aiProvider, preferences.aiModel));
  }

  private static noProviders(): ConfiguredProviders {
    return Object.fromEntries(AI_PROVIDERS.map((provider) => [provider, false])) as ConfiguredProviders;
  }
}
