import { z } from "zod";
import type { StructuredModel } from "./contracts";
import type { HarnessLanguage } from "./harness/HarnessPieces";
import { TikiHarness } from "./harness/TikiHarness";
import { ProposedActions, type ProposedAction } from "./ProposedAction";

/** One open item as the advisor receives it (see `AssistantContext` in the core). */
export interface AdvisorTask {
  taskId: number;
  kind: string;
  title: string;
  projectName: string | null;
  tags: string[];
  status: string;
  priority: string;
  dueDate: string | null;
  remindAt: string | null;
  waitingDays: number | null;
}

export interface AdvisorContext {
  /** Local date and time, for example `2026-10-21T17:30:00`. */
  now: string;
  openTasks: AdvisorTask[];
  /** Projects with the notes the user wrote for them. */
  projects: Array<{ name: string; notes: string }>;
}

/** The language Tiki answers in, as the model should read it. */
export type AdvisorLanguage = HarnessLanguage;

/** An answer, with the changes the assistant proposes (possibly none). */
export interface AskReply {
  text: string;
  actions: ProposedAction[];
}

/** Summarizes the user's day and answers questions, proposing actions the user approves. */
export class DayAdvisor {
  private static readonly briefSchema = z.object({ text: z.string() });
  private static readonly answerSchema = z.object({
    text: z.string(),
    actions: z.array(ProposedActions.schema),
  });

  constructor(private readonly model: StructuredModel) {}

  /** A short brief: what to start with, what is at risk, reminders and who to follow up. */
  async summarizeDay(context: AdvisorContext, language: AdvisorLanguage): Promise<string> {
    const answer = await this.model.extract({
      instructions: TikiHarness.brief(language, context.projects),
      input: DayAdvisor.contextText(context),
      schema: DayAdvisor.briefSchema,
      schemaName: "day_brief",
    });
    return answer.text.trim();
  }

  /** Answers a question about the user's work; proposes only actions Tiki can apply. */
  async ask(question: string, context: AdvisorContext, language: AdvisorLanguage): Promise<AskReply> {
    const answer = await this.model.extract({
      instructions: TikiHarness.answer(language, context.projects),
      input: `${DayAdvisor.contextText(context)}\n\nQuestion: ${question}`,
      schema: DayAdvisor.answerSchema,
      schemaName: "tiki_answer",
    });
    const taskIds = new Set(context.openTasks.map((task) => task.taskId));
    const projectNames = context.projects.map((project) => project.name)
      .concat(context.openTasks.flatMap((task) => (task.projectName ? [task.projectName] : [])));
    return {
      text: answer.text.trim(),
      actions: answer.actions
        .map((action) => ProposedActions.validate(action, taskIds, projectNames))
        .filter((action): action is ProposedAction => action !== null),
    };
  }

  private static contextText(context: AdvisorContext): string {
    return `Now is ${context.now}. Open items (JSON):\n${JSON.stringify(context.openTasks)}`;
  }
}
