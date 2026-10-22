import { z } from "zod";
import { AiDates } from "./AiDates";
import type { StructuredModel } from "./contracts";
import type { AdvisorContext } from "./DayAdvisor";
import type { HarnessLanguage, HarnessLesson } from "./harness/HarnessPieces";
import { TikiHarness } from "./harness/TikiHarness";
import { ProposedActions, type ProposedAction } from "./ProposedAction";

const PRIORITIES = ["low", "normal", "high", "urgent"] as const;
const KINDS = ["task", "reminder"] as const;
const REPEATS = ["never", "daily", "weekdays", "weekly", "monthly"] as const;

/** An item read from what the user wrote, ready for them to review before saving. */
export interface InterpretedCapture {
  title: string;
  /** A known project, or a new one the user named (it is created when saved). */
  projectName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: (typeof PRIORITIES)[number];
  /** When to remind, for example `2026-10-23T09:00:00`. */
  remindAt: string | null;
  kind: (typeof KINDS)[number];
  tags: string[];
  /** How often a reminder comes back. */
  repeat: (typeof REPEATS)[number];
  /** Who or what a task waits for, as the user said it; the task then waits. */
  waitingFor: string | null;
}

export interface CommandContext {
  now: Date;
  language: HarnessLanguage;
  /** Open items and project notes, so questions and changes can refer to them. */
  advisor: AdvisorContext;
  /** The user's recent corrections, so the AI follows their way of saying things. */
  lessons: readonly HarnessLesson[];
}

/** What the user's words were: something to keep, or a question or change request. */
export type CommandReply =
  | { kind: "capture"; item: InterpretedCapture }
  | { kind: "answer"; text: string; actions: ProposedAction[] };

/**
 * The single box of Tiki: one call reads what the user wrote and decides whether it is an
 * item to keep (filling its fields) or a question or a change (answering it and proposing
 * actions the user approves).
 */
export class CommandInterpreter {
  private static readonly itemSchema = z.object({
    kind: z.enum(KINDS),
    title: z.string(),
    projectName: z.string().nullable(),
    dueDate: z.string().nullable(),
    remindAt: z.string().nullable(),
    repeat: z.enum(REPEATS),
    waitingFor: z.string().nullable(),
    tags: z.array(z.string()),
    priority: z.enum(PRIORITIES),
  });

  private static readonly schema = z.object({
    intent: z.enum(["capture", "answer"]),
    item: CommandInterpreter.itemSchema.nullable(),
    answer: z.string().nullable(),
    actions: z.array(ProposedActions.schema),
  });

  constructor(private readonly model: StructuredModel) {}

  async read(text: string, context: CommandContext): Promise<CommandReply> {
    const projectNames = CommandInterpreter.projectNames(context.advisor);
    const reply = await this.model.extract({
      instructions: TikiHarness.command(context.language, context.now, projectNames, context.advisor.projects, context.lessons),
      input: `Now is ${context.advisor.now}. Open items (JSON):\n${JSON.stringify(context.advisor.openTasks)}\n\nThe user wrote: ${text}`,
      schema: CommandInterpreter.schema,
      schemaName: "tiki_command",
    });
    if (reply.intent === "capture" && reply.item) {
      return { kind: "capture", item: CommandInterpreter.normalize(reply.item, projectNames) };
    }
    const taskIds = new Set(context.advisor.openTasks.map((task) => task.taskId));
    return {
      kind: "answer",
      text: reply.answer?.trim() ?? "",
      actions: reply.actions
        .map((action) => ProposedActions.validate(action, taskIds, projectNames))
        .filter((action): action is ProposedAction => action !== null),
    };
  }

  private static normalize(item: z.infer<typeof CommandInterpreter.itemSchema>, projectNames: readonly string[]): InterpretedCapture {
    const remindAt = AiDates.dateTime(item.remindAt);
    // A reminder without a moment is not a reminder: keep it as a task.
    const kind = item.kind === "reminder" && remindAt ? "reminder" : "task";
    return {
      title: item.title.trim(),
      projectName: CommandInterpreter.project(item.projectName, projectNames),
      dueDate: kind === "reminder" ? null : AiDates.date(item.dueDate),
      priority: item.priority,
      remindAt,
      kind,
      tags: item.tags.map((tag) => tag.trim().replace(/^#/, "").toLowerCase()).filter((tag) => tag !== ""),
      repeat: kind === "reminder" ? item.repeat : "never",
      waitingFor: kind === "task" ? item.waitingFor?.trim() || null : null,
    };
  }

  /** A known project written as the user saved it; otherwise the new name, trimmed. */
  private static project(name: string | null, projectNames: readonly string[]): string | null {
    const trimmed = name?.trim();
    if (!trimmed) {
      return null;
    }
    return projectNames.find((known) => known.toLowerCase() === trimmed.toLowerCase()) ?? trimmed;
  }

  private static projectNames(context: AdvisorContext): string[] {
    const names = new Set([
      ...context.projects.map((project) => project.name),
      ...context.openTasks.flatMap((task) => (task.projectName ? [task.projectName] : [])),
    ]);
    return [...names];
  }
}
