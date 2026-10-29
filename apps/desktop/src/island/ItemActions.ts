import type { ProjectChange, TaskEdit, TikiBackend } from "../backend/contracts";
import type { IslandStore } from "./IslandStore";

/** What the user does to items and projects: open, save, deliver, snooze, join, manage. */
export class ItemActions {
  constructor(
    private readonly backend: TikiBackend,
    private readonly store: IslandStore,
  ) {}

  /** Opens a task in the edit panel. */
  async open(taskId: number): Promise<void> {
    const task = await this.backend.taskDetails(taskId);
    this.store.update({ mode: "expanded", panel: "edit", editing: task });
  }

  async save(taskId: number, edit: TaskEdit): Promise<void> {
    await this.backend.editTask(taskId, edit);
    await this.store.refresh();
    this.store.update({ panel: "tabs", editing: null });
    this.store.react(edit.status === "done" ? "happy" : "curious");
  }

  async deliver(taskId: number): Promise<void> {
    await this.backend.moveTask(taskId, "done");
    await this.store.refresh();
    this.store.react("happy");
  }

  async reopen(taskId: number): Promise<void> {
    await this.backend.moveTask(taskId, "to_do");
    await this.store.refresh();
  }

  /** Puts off the reminder and puts the alert away. */
  async snooze(taskId: number, minutes: number): Promise<void> {
    await this.backend.snooze(taskId, minutes);
    this.store.update({ alert: null, unreadAlert: false });
    await this.store.refresh();
  }

  /** Opens the video call and puts the meeting alert away. */
  async joinMeeting(url: string): Promise<void> {
    this.store.update({ alert: null, unreadAlert: false });
    await this.backend.openMeetingLink(url);
  }

  createProject(name: string): Promise<void> {
    return this.changeProjects(() => this.backend.createProject(name));
  }

  changeProject(projectId: number, change: ProjectChange): Promise<void> {
    return this.changeProjects(() => this.backend.changeProject(projectId, change));
  }

  removeProject(projectId: number): Promise<void> {
    return this.changeProjects(() => this.backend.removeProject(projectId));
  }

  /** Creates, changes or removes a project, then shows the result; a taken name is ignored. */
  private async changeProjects(action: () => Promise<unknown>): Promise<void> {
    try {
      await action();
    } catch {
      this.store.react("worried");
    }
    await this.store.refresh();
  }
}
