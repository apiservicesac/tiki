import type { z } from "zod";

export type AiProviderName = "anthropic" | "openai";

export interface AiModelOption {
  id: string;
  label: string;
}

/** One structured question for a model: instructions, the user's text, and the answer's shape. */
export interface ExtractionTask<Answer> {
  instructions: string;
  input: string;
  schema: z.ZodType<Answer>;
  /** Short identifier of the schema, required by some providers. */
  schemaName: string;
}

/** What Tiki needs from any AI provider. Each provider wraps its official SDK. */
export interface StructuredModel {
  extract<Answer>(task: ExtractionTask<Answer>): Promise<Answer>;
}

/** A request built by an official SDK, sent by the desktop shell with the key attached. */
export interface ProxiedRequest {
  url: string;
  method: string;
  headers: Array<[string, string]>;
  body: string | null;
}

export interface ProxiedResponse {
  status: number;
  headers: Array<[string, string]>;
  body: string;
}

export type SendThroughShell = (request: ProxiedRequest) => Promise<ProxiedResponse>;
