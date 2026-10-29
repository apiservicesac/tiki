import { describe, expect, it } from "vitest";
import { BackendError } from "../src/backend/BackendError";

describe("BackendError", () => {
  it("reads the key the shell sends with a refused command", () => {
    const error = BackendError.from({ key: "reminder_needs_time", message: "a reminder needs a time to ring" });
    expect(error).toBeInstanceOf(BackendError);
    expect((error as BackendError).key).toBe("reminder_needs_time");
  });

  it("names the message of a failure the user can fix, and a general one otherwise", () => {
    expect(BackendError.messageKeyOf(new BackendError("reminder_needs_time", ""))).toBe("error.reminder_needs_time");
    expect(BackendError.messageKeyOf(new BackendError("database", ""))).toBe("error.save_failed");
    expect(BackendError.messageKeyOf(new Error("offline"))).toBe("error.save_failed");
  });
});
