/**
 * Tiki's internal AI library. The rest of the app imports only from here; the official
 * SDKs (Anthropic, OpenAI) stay behind `TikiAi`, so adding or swapping a provider never
 * touches the island.
 */
export { AiError } from "./AiError";
export { CommandInterpreter, type CommandContext, type CommandReply, type InterpretedCapture } from "./CommandInterpreter";
export { DayAdvisor, type AdvisorContext, type AdvisorLanguage, type AdvisorTask } from "./DayAdvisor";
export type { HarnessLesson } from "./harness/HarnessPieces";
export { ACTION_TYPES, type ProposedAction } from "./ProposedAction";
export {
  SessionNotesAdvisor,
  type NotesLine,
  type ProposedSessionTask,
  type SessionNotesContext,
  type WrittenSessionNotes,
} from "./SessionNotesAdvisor";
export type { AiModelOption, AiProviderName, ProxiedRequest, ProxiedResponse, StructuredModel } from "./contracts";
export { TikiAi } from "./TikiAi";
