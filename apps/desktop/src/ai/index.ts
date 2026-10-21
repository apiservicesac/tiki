/**
 * Tiki's internal AI library. The rest of the app imports only from here; the official
 * SDKs (Anthropic, OpenAI) stay behind `TikiAi`, so adding or swapping a provider never
 * touches the island.
 */
export { AiError } from "./AiError";
export { CaptureInterpreter, type CaptureContext, type InterpretedCapture } from "./CaptureInterpreter";
export type { AiModelOption, AiProviderName, ProxiedRequest, ProxiedResponse, StructuredModel } from "./contracts";
export { TikiAi } from "./TikiAi";
