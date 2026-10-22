import { z } from "zod";
import { AiDates } from "./AiDates";

export const ACTION_TYPES = Object.freeze(["create_task", "create_reminder", "set_reminder", "set_due_date", "move_task"] as const);
const STATUSES = ["to_do", "in_progress", "waiting", "done"] as const;

/** A change the assistant proposes; nothing happens until the user applies it. */
export interface ProposedAction {
  type: (typeof ACTION_TYPES)[number];
  /** One line in the user's language describing the change. */
  summary: string;
  taskId: number | null;
  title: string | null;
  projectName: string | null;
  dueDate: string | null;
  /** For example `2026-10-23T09:00:00`. */
  remindAt: string | null;
  status: (typeof STATUSES)[number] | null;
}

/** The shape the model fills in, and the checks that keep only actions Tiki can apply. */
export class ProposedActions {
  static readonly schema = z.object({
    type: z.enum(ACTION_TYPES),
    summary: z.string(),
    taskId: z.number().int().nullable(),
    title: z.string().nullable(),
    projectName: z.string().nullable(),
    dueDate: z.string().nullable(),
    remindAt: z.string().nullable(),
    status: z.enum(STATUSES).nullable(),
  });

  /**
   * Keeps an action only when it has what it needs: an existing task for changes, a title
   * for new items, a moment for reminders and a date for due dates.
   */
  static validate(
    raw: z.infer<typeof ProposedActions.schema>,
    taskIds: ReadonlySet<number>,
    projectNames: readonly string[],
  ): ProposedAction | null {
    const action: ProposedAction = {
      ...raw,
      summary: raw.summary.trim(),
      title: raw.title?.trim() || null,
      projectName: projectNames.find((name) => name.toLowerCase() === raw.projectName?.trim().toLowerCase()) ?? null,
      dueDate: AiDates.date(raw.dueDate),
      remindAt: AiDates.dateTime(raw.remindAt),
    };
    const knownTask = action.taskId !== null && taskIds.has(action.taskId);
    switch (action.type) {
      case "create_task":
        return action.title ? action : null;
      case "create_reminder":
        return action.title && action.remindAt ? action : null;
      case "set_reminder":
        return knownTask && action.remindAt ? action : null;
      case "set_due_date":
        return knownTask && action.dueDate ? action : null;
      case "move_task":
        return knownTask && action.status ? action : null;
    }
  }
}
