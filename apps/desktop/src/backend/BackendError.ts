import { USER_ERROR_KEYS, type CommandFailure } from "./contracts";

/** A command the core or the shell refused, with the stable key of the failure. */
export class BackendError extends Error {
  /** The message for a failure the user cannot fix from the form. */
  private static readonly SAVE_FAILED_KEY = "error.save_failed";

  constructor(
    readonly key: string,
    message: string,
  ) {
    super(message);
    this.name = "BackendError";
  }

  /** What a command was rejected with: the shell's `CommandError` becomes a `BackendError`. */
  static from(failure: unknown): unknown {
    return BackendError.isCommandFailure(failure) ? new BackendError(failure.key, failure.message) : failure;
  }

  /**
   * The catalog key of the message to show when saving failed: the failure's own text for one
   * the user can fix (`error.reminder_needs_time`), a general one for anything else.
   */
  static messageKeyOf(error: unknown): string {
    const fixable = error instanceof BackendError && USER_ERROR_KEYS.some((key) => key === error.key);
    return fixable ? `error.${error.key}` : BackendError.SAVE_FAILED_KEY;
  }

  private static isCommandFailure(failure: unknown): failure is CommandFailure {
    return typeof failure === "object"
      && failure !== null
      && typeof (failure as Partial<CommandFailure>).key === "string"
      && typeof (failure as Partial<CommandFailure>).message === "string";
  }
}
