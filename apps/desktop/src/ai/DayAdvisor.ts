import { z } from "zod";
import type { StructuredModel } from "./contracts";

/** One open task as the advisor receives it (see `AssistantContext` in the core). */
export interface AdvisorTask {
  title: string;
  clientName: string | null;
  status: string;
  priority: string;
  dueDate: string | null;
  waitingDays: number | null;
}

export interface AdvisorContext {
  /** ISO date, for example `2026-10-21`. */
  today: string;
  openTasks: AdvisorTask[];
}

/** The language Tiki answers in, as the model should read it. */
export type AdvisorLanguage = "Spanish" | "English";

/** Summarizes the user's day and answers questions about their pending work. */
export class DayAdvisor {
  private static readonly schema = z.object({ text: z.string() });

  constructor(private readonly model: StructuredModel) {}

  /** A short brief: what to start with, what is at risk and who to follow up. */
  async summarizeDay(context: AdvisorContext, language: AdvisorLanguage): Promise<string> {
    return this.ask(
      [
        "You are Tiki, a friendly desk assistant for a support consultant.",
        `Write their brief for today in ${language}: two to four short sentences, plain text, no lists or markdown.`,
        "Say what to start with and why, mention anything overdue or due soon by client, and who to follow up when something has waited three days or more.",
        "When nothing is pressing, say so warmly in one sentence.",
      ],
      DayAdvisor.contextText(context),
      "day_brief",
    );
  }

  /** Answers a question using only the open tasks it was given. */
  async answer(question: string, context: AdvisorContext, language: AdvisorLanguage): Promise<string> {
    return this.ask(
      [
        "You are Tiki, a friendly desk assistant for a support consultant.",
        `Answer their question in ${language} in at most four short sentences, plain text, no markdown.`,
        "Use only the open tasks listed. When the answer is not in them, say you do not know yet.",
        "You cannot change tasks; when asked to, say how they can do it in Tiki.",
      ],
      `${DayAdvisor.contextText(context)}\n\nQuestion: ${question}`,
      "tiki_answer",
    );
  }

  private async ask(instructions: string[], input: string, schemaName: string): Promise<string> {
    const answer = await this.model.extract({
      instructions: instructions.join("\n"),
      input,
      schema: DayAdvisor.schema,
      schemaName,
    });
    return answer.text.trim();
  }

  private static contextText(context: AdvisorContext): string {
    return `Today is ${context.today}. Open tasks (JSON):\n${JSON.stringify(context.openTasks)}`;
  }
}
