import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SHELL_EVENTS } from "../config/uiConfig";
import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../ai";
import type {
  AiProviderStatus,
  AvailableUpdate,
  CalendarAccount,
  CalendarProvider,
  CalendarStatus,
  CaptureLesson,
  ProjectChange,
  ProjectSummary,
  DeliveredItem,
  TikiAlert,
  HitArea,
  IslandSnapshot,
  TaskDetails,
  TaskEdit,
  TaskStatus,
  TikiBackend,
  UserPreferences,
} from "./contracts";

/** Talks to the Rust core through Tauri commands (see `src-tauri/src/commands`). */
export class TauriBackend implements TikiBackend {
  islandSnapshot(): Promise<IslandSnapshot> {
    return invoke<IslandSnapshot>("island_snapshot");
  }

  async moveTask(taskId: number, status: TaskStatus): Promise<void> {
    await invoke("move_task", { taskId, status });
  }

  async snooze(taskId: number, minutes: number): Promise<void> {
    await invoke("snooze", { taskId, minutes });
  }

  preferences(): Promise<UserPreferences> {
    return invoke<UserPreferences>("preferences");
  }

  async updatePreferences(preferences: UserPreferences): Promise<void> {
    await invoke("update_preferences", { preferences });
  }

  async saveConfirmedCapture(capture: InterpretedCapture): Promise<void> {
    await invoke("save_confirmed_capture", { capture });
  }

  projectNames(): Promise<string[]> {
    return invoke<string[]>("project_names");
  }

  taskDetails(taskId: number): Promise<TaskDetails> {
    return invoke<TaskDetails>("task_details", { taskId });
  }

  projectSummaries(): Promise<ProjectSummary[]> {
    return invoke<ProjectSummary[]>("project_summaries");
  }

  createProject(name: string): Promise<number> {
    return invoke<number>("create_project", { name });
  }

  async changeProject(projectId: number, change: ProjectChange): Promise<void> {
    await invoke("change_project", { projectId, change });
  }

  async removeProject(projectId: number): Promise<void> {
    await invoke("remove_project", { projectId });
  }

  projectColors(): Promise<string[]> {
    return invoke<string[]>("project_colors");
  }

  deliveredTasks(): Promise<DeliveredItem[]> {
    return invoke<DeliveredItem[]>("delivered_tasks");
  }

  captureLessons(): Promise<CaptureLesson[]> {
    return invoke<CaptureLesson[]>("capture_lessons");
  }

  async recordCaptureLesson(lesson: CaptureLesson): Promise<void> {
    await invoke("record_capture_lesson", { lesson });
  }

  assistantContext(): Promise<AdvisorContext> {
    return invoke<AdvisorContext>("assistant_context");
  }

  async editTask(taskId: number, edit: TaskEdit): Promise<void> {
    await invoke("edit_task", { taskId, edit });
  }

  aiStatus(): Promise<AiProviderStatus[]> {
    return invoke<AiProviderStatus[]>("ai_status");
  }

  async saveAiKey(provider: AiProviderName, apiKey: string): Promise<void> {
    await invoke("save_ai_key", { provider, apiKey });
  }

  async removeAiKey(provider: AiProviderName): Promise<void> {
    await invoke("remove_ai_key", { provider });
  }

  aiFetch(request: ProxiedRequest): Promise<ProxiedResponse> {
    return invoke<ProxiedResponse>("ai_fetch", { request });
  }

  calendarStatus(): Promise<CalendarStatus> {
    return invoke<CalendarStatus>("calendar_status");
  }

  connectCalendar(provider: CalendarProvider): Promise<CalendarAccount> {
    return invoke<CalendarAccount>("connect_calendar", { provider });
  }

  async disconnectCalendar(accountId: number): Promise<void> {
    await invoke("disconnect_calendar", { accountId });
  }

  async syncCalendars(): Promise<void> {
    await invoke("sync_calendars");
  }

  async openMeetingLink(url: string): Promise<void> {
    await invoke("open_meeting_link", { url });
  }

  async openAdvancedSettings(section?: string): Promise<void> {
    await invoke("open_advanced_settings", { section: section ?? null });
  }

  autostartEnabled(): Promise<boolean> {
    return invoke<boolean>("autostart_enabled");
  }

  appVersion(): Promise<string> {
    return invoke<string>("app_version");
  }

  checkForUpdate(): Promise<AvailableUpdate | null> {
    return invoke<AvailableUpdate | null>("check_for_update");
  }

  async installUpdate(): Promise<void> {
    await invoke("install_update");
  }

  async setAutostart(enabled: boolean): Promise<void> {
    await invoke("set_autostart", { enabled });
  }

  async setHitArea(area: HitArea): Promise<void> {
    await invoke("set_island_hit_area", { area });
  }

  onAlert(handler: (alert: TikiAlert) => void): Promise<() => void> {
    return listen<TikiAlert>(SHELL_EVENTS.alertReady, (event) => handler(event.payload));
  }

  listen(eventName: string, handler: () => void): Promise<() => void> {
    return listen(eventName, handler);
  }
}
