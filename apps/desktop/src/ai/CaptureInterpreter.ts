import { z } from "zod";
import { AI_CONFIG } from "./aiConfig";
import type { StructuredModel } from "./contracts";

const PRIORITIES = ["low", "normal", "high", "urgent"] as const;

/** A task read from a sentence, ready for the user to confirm. */
export interface InterpretedCapture {
  title: string;
  projectName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: (typeof PRIORITIES)[number];
  /** When to remind, for example `2026-10-23T09:00:00`; `null` when not asked. */
  remindAt: string | null;
}

export interface CaptureContext {
  /** Projects the user already has; the answer may only pick one of these. */
  projectNames: readonly string[];
  today: Date;
}

/** Turns a free sentence ("revisar con Almendra el acceso antes del viernes") into a task. */
export class CaptureInterpreter {
  private static readonly schema = z.object({
    title: z.string(),
    projectName: z.string().nullable(),
    dueDate: z.string().nullable(),
    priority: z.enum(PRIORITIES),
    remindAt: z.string().nullable(),
  });

  constructor(private readonly model: StructuredModel) {}

  async interpret(sentence: string, context: CaptureContext): Promise<InterpretedCapture> {
    const answer = await this.model.extract({
      instructions: CaptureInterpreter.instructions(context),
      input: sentence,
      schema: CaptureInterpreter.schema,
      schemaName: "captured_task",
    });
    return {
      title: answer.title.trim(),
      projectName: CaptureInterpreter.knownProject(answer.projectName, context.projectNames),
      dueDate: answer.dueDate && /^\d{4}-\d{2}-\d{2}$/.test(answer.dueDate) ? answer.dueDate : null,
      priority: answer.priority,
      remindAt: answer.remindAt && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(answer.remindAt) ? `${answer.remindAt}:00` : null,
    };
  }

  /** Only names Tiki already knows, written exactly as the user saved them. */
  private static knownProject(name: string | null, projectNames: readonly string[]): string | null {
    if (!name) {
      return null;
    }
    return projectNames.find((known) => known.toLowerCase() === name.trim().toLowerCase()) ?? null;
  }

  private static instructions(context: CaptureContext): string {
    const today = context.today;
    const isoToday = [
      today.getFullYear(),
      String(today.getMonth() + 1).padStart(2, "0"),
      String(today.getDate()).padStart(2, "0"),
    ].join("-");
    const weekday = today.toLocaleDateString("en-US", { weekday: "long" });
    const time = [today.getHours(), today.getMinutes()].map((part) => String(part).padStart(2, "0")).join(":");
    const projects = context.projectNames.length > 0 ? context.projectNames.join(", ") : "none yet";
    return [
      "You turn one sentence a person typed into a task for their to-do list. They may write it any way they like.",
      `Now is ${weekday}, ${isoToday} ${time}. Known projects: ${projects}.`,
      "- title: a short task title in the same language as the sentence, without the project name and without the date words.",
      "- projectName: one of the known projects, written exactly as listed, or null when none is mentioned.",
      "- dueDate: YYYY-MM-DD when the sentence mentions a day, said any way (tomorrow, Friday, the 25th, in two days, next week, end of the month); resolve it against now (a weekday means its next occurrence). Otherwise null.",
      `- remindAt: YYYY-MM-DDTHH:MM when the sentence asks to be reminded (remind me, avísame, recuérdame) or says a time; use the due day, or today, and ${AI_CONFIG.defaultReminderTime} when no time is given; if that moment has already passed and no day was named, use tomorrow. Otherwise null. The words of the reminder are not part of the title.`,
      "- priority: urgent only when the sentence says it is urgent; high for words like important or as soon as possible; low when it says there is no hurry; otherwise normal.",
    ].join("\n");
  }
}
