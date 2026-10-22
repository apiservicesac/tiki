import { z } from "zod";
import { AiDates } from "./AiDates";
import type { StructuredModel } from "./contracts";
import type { HarnessLesson } from "./harness/HarnessPieces";
import { TikiHarness } from "./harness/TikiHarness";

const PRIORITIES = ["low", "normal", "high", "urgent"] as const;
const KINDS = ["task", "reminder"] as const;

/** An item read from a sentence, ready for the user to review. */
export interface InterpretedCapture {
  title: string;
  projectName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: (typeof PRIORITIES)[number];
  /** When to remind, for example `2026-10-23T09:00:00`; `null` when not asked. */
  remindAt: string | null;
  /** A task, or a standalone reminder. */
  kind: (typeof KINDS)[number];
  tags: string[];
}

export interface CaptureContext {
  /** Projects the user already has; the answer may only pick one of these. */
  projectNames: readonly string[];
  now: Date;
  /** The user's recent corrections, so the AI follows their way of saying things. */
  lessons: readonly HarnessLesson[];
}

/** Turns a free sentence ("revisar con Almendra el acceso antes del viernes") into an item. */
export class CaptureInterpreter {
  private static readonly schema = z.object({
    kind: z.enum(KINDS),
    title: z.string(),
    projectName: z.string().nullable(),
    dueDate: z.string().nullable(),
    remindAt: z.string().nullable(),
    tags: z.array(z.string()),
    priority: z.enum(PRIORITIES),
  });

  constructor(private readonly model: StructuredModel) {}

  async interpret(sentence: string, context: CaptureContext): Promise<InterpretedCapture> {
    const answer = await this.model.extract({
      instructions: TikiHarness.capture(context.now, context.projectNames, context.lessons),
      input: sentence,
      schema: CaptureInterpreter.schema,
      schemaName: "captured_item",
    });
    const remindAt = AiDates.dateTime(answer.remindAt);
    // A reminder without a moment is not a reminder: keep it as a task.
    const kind = answer.kind === "reminder" && remindAt ? "reminder" : "task";
    return {
      title: answer.title.trim(),
      projectName: CaptureInterpreter.knownProject(answer.projectName, context.projectNames),
      dueDate: kind === "reminder" ? null : AiDates.date(answer.dueDate),
      priority: answer.priority,
      remindAt,
      kind,
      tags: answer.tags.map((tag) => tag.trim().replace(/^#/, "").toLowerCase()).filter((tag) => tag !== ""),
    };
  }

  /** Only names Tiki already knows, written exactly as the user saved them. */
  private static knownProject(name: string | null, projectNames: readonly string[]): string | null {
    if (!name) {
      return null;
    }
    return projectNames.find((known) => known.toLowerCase() === name.trim().toLowerCase()) ?? null;
  }
}
