import {
  CommandInterpreter,
  DayAdvisor,
  SessionNotesAdvisor,
  TikiAi,
  type AdvisorLanguage,
  type AiModelOption,
  type AiProviderName,
  type CommandReply,
  type InterpretedCapture,
  type WrittenSessionNotes,
} from "../ai";
import { AI_PROVIDERS, type LocaleName, type SessionDetails, type TikiBackend, type UserPreferences } from "../backend/contracts";

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

  /**
   * Reads what the user wrote in the single box: an item to keep (with its fields), or a
   * question or change request (with an answer and proposed actions). Sends the text, the
   * open items, project notes and the user's recent corrections.
   */
  async command(text: string, preferences: UserPreferences): Promise<CommandReply> {
    const [advisor, lessons] = await Promise.all([this.backend.assistantContext(), this.backend.captureLessons()]);
    const interpreter = new CommandInterpreter(this.ai.model(preferences.aiProvider, preferences.aiModel));
    return interpreter.read(text, { now: new Date(), language: ADVISOR_LANGUAGES[preferences.locale], advisor, lessons });
  }

  /** When the user changed what the AI understood before saving, Tiki remembers it. */
  async learn(sentence: string, understood: InterpretedCapture, saved: InterpretedCapture): Promise<void> {
    if (JSON.stringify(understood) !== JSON.stringify(saved)) {
      await this.backend.recordCaptureLesson({ sentence, understood, corrected: saved });
    }
  }

  /** A short AI brief of the user's open tasks. */
  async summarizeDay(preferences: UserPreferences): Promise<string> {
    const context = await this.backend.assistantContext();
    return this.advisor(preferences).summarizeDay(context, ADVISOR_LANGUAGES[preferences.locale]);
  }

  /** The notes of a session from its transcript, with the projects' notes as context. */
  async sessionNotes(details: SessionDetails, preferences: UserPreferences): Promise<WrittenSessionNotes> {
    const context = await this.backend.assistantContext();
    const advisor = new SessionNotesAdvisor(this.ai.model(preferences.aiProvider, preferences.aiModel));
    return advisor.write(details.transcript, {
      title: details.summary.title,
      startedAt: details.summary.startedAt,
      language: ADVISOR_LANGUAGES[preferences.locale],
      projects: context.projects,
    });
  }

  private advisor(preferences: UserPreferences): DayAdvisor {
    return new DayAdvisor(this.ai.model(preferences.aiProvider, preferences.aiModel));
  }

  private static noProviders(): ConfiguredProviders {
    return Object.fromEntries(AI_PROVIDERS.map((provider) => [provider, false])) as ConfiguredProviders;
  }
}
