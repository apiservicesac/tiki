import type { ProposedAction } from "../ai";
import type { TaskDetails, TaskEdit, TikiBackend } from "../backend/contracts";

/** Applies an action the assistant proposed, once the user approved it. */
export class ActionApplier {
  constructor(private readonly backend: TikiBackend) {}

  async apply(action: ProposedAction): Promise<void> {
    switch (action.type) {
      case "create_task":
      case "create_reminder":
        await this.backend.saveConfirmedCapture({
          title: action.title ?? "",
          projectName: action.projectName,
          dueDate: action.type === "create_task" ? action.dueDate : null,
          priority: "normal",
          remindAt: action.remindAt,
          kind: action.type === "create_task" ? "task" : "reminder",
          tags: [],
          repeat: "never",
          waitingFor: null,
        });
        return;
      case "set_reminder":
        await this.change(action, (edit) => ({ ...edit, remindAt: action.remindAt }));
        return;
      case "set_due_date":
        await this.change(action, (edit) => ({ ...edit, dueDate: action.dueDate }));
        return;
      case "move_task":
        if (action.taskId !== null && action.status !== null) {
          await this.backend.moveTask(action.taskId, action.status);
        }
        return;
    }
  }

  /** Changes one field of a saved item, keeping the rest as it is. */
  private async change(action: ProposedAction, update: (edit: TaskEdit) => TaskEdit): Promise<void> {
    if (action.taskId === null) {
      return;
    }
    const details = await this.backend.taskDetails(action.taskId);
    await this.backend.editTask(action.taskId, update(ActionApplier.editOf(details)));
  }

  private static editOf(details: TaskDetails): TaskEdit {
    const { taskId: _taskId, ...edit } = details;
    return edit;
  }
}
