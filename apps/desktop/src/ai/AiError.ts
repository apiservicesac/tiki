export type AiErrorReason = "refused" | "incomplete" | "invalid_answer";

/** The model answered, but not with something Tiki can use. Network and API errors keep the SDK's own classes. */
export class AiError extends Error {
  constructor(
    readonly reason: AiErrorReason,
    message: string,
  ) {
    super(message);
    this.name = "AiError";
  }
}
