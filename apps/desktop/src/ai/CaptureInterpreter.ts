import { z } from "zod";
import { AI_CONFIG } from "./aiConfig";
import type { StructuredModel } from "./contracts";

const PRIORITIES = ["low", "normal", "high", "urgent"] as const;

/** A task read from a sentence, ready for the user to confirm. */
export interface InterpretedCapture {
  title: string;
  clientName: string | null;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
  priority: (typeof PRIORITIES)[number];
  /** When to remind, for example `2026-10-23T09:00:00`; `null` when not asked. */
  remindAt: string | null;
}

export interface CaptureContext {
  /** Clients the user already has; the answer may only pick one of these. */
  clientNames: readonly string[];
  today: Date;
}

/** Turns a free sentence ("revisar con Almendra el acceso antes del viernes") into a task. */
export class CaptureInterpreter {
  private static readonly schema = z.object({
    title: z.string(),
    clientName: z.string().nullable(),
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
      clientName: CaptureInterpreter.knownClient(answer.clientName, context.clientNames),
      dueDate: answer.dueDate && /^\d{4}-\d{2}-\d{2}$/.test(answer.dueDate) ? answer.dueDate : null,
      priority: answer.priority,
      remindAt: answer.remindAt && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/.test(answer.remindAt) ? `${answer.remindAt}:00` : null,
    };
  }

  /** Only names Tiki already knows, written exactly as the user saved them. */
  private static knownClient(name: string | null, clientNames: readonly string[]): string | null {
    if (!name) {
      return null;
    }
    return clientNames.find((known) => known.toLowerCase() === name.trim().toLowerCase()) ?? null;
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
    const clients = context.clientNames.length > 0 ? context.clientNames.join(", ") : "none yet";
    return [
      "You turn one sentence that a support consultant typed into a task for their to-do list.",
      `Now is ${weekday}, ${isoToday} ${time}. Known clients: ${clients}.`,
      "- title: a short task title in the same language as the sentence, without the client name and without the date words.",
      "- clientName: one of the known clients, written exactly as listed, or null when none is mentioned.",
      "- dueDate: YYYY-MM-DD when the sentence mentions a day; resolve words like tomorrow or Friday against today (a weekday means its next occurrence). Otherwise null.",
      `- remindAt: YYYY-MM-DDTHH:MM when the sentence asks to be reminded (remind me, avísame, recuérdame) or says a time; use the due day, or today, and ${AI_CONFIG.defaultReminderTime} when no time is given; if that moment has already passed and no day was named, use tomorrow. Otherwise null. The words of the reminder are not part of the title.`,
      "- priority: urgent only when the sentence says it is urgent; high for words like important or as soon as possible; low when it says there is no hurry; otherwise normal.",
    ].join("\n");
  }
}
