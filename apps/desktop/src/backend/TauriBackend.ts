import { convertFileSrc, invoke, type InvokeArgs } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SHELL_EVENTS } from "../config/uiConfig";
import { BackendError } from "./BackendError";
import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse, WrittenSessionNotes } from "../ai";
import type {
  AiProviderStatus,
  AvailableUpdate,
  CalendarAccount,
  CaptureTarget,
  CalendarProvider,
  CalendarStatus,
  CaptureLesson,
  ModelDownload,
  RecordingMode,
  SessionDetails,
  SessionSearchHit,
  SessionStorage,
  SessionSummary,
  DeliveredItem,
  DueAlert,
  HitArea,
  IslandSnapshot,
  ProjectChange,
  ProjectSummary,
  SystemStats,
  TaskDetails,
  TaskEdit,
  TaskStatus,
  TikiBackend,
  TrashedItem,
  UserPreferences,
  VideoChoice,
} from "./contracts";

/** Talks to the Rust core through Tauri commands (see `src-tauri/src/commands`). */
export class TauriBackend implements TikiBackend {
  islandSnapshot(): Promise<IslandSnapshot> {
    return this.call<IslandSnapshot>("island_snapshot");
  }

  async moveTask(taskId: number, status: TaskStatus): Promise<void> {
    await this.call("move_task", { taskId, status });
  }

  async snooze(taskId: number, minutes: number): Promise<void> {
    await this.call("snooze", { taskId, minutes });
  }

  preferences(): Promise<UserPreferences> {
    return this.call<UserPreferences>("preferences");
  }

  async updatePreferences(preferences: UserPreferences): Promise<void> {
    await this.call("update_preferences", { preferences });
  }

  async saveConfirmedCapture(capture: InterpretedCapture): Promise<void> {
    await this.call("save_confirmed_capture", { capture });
  }

  projectNames(): Promise<string[]> {
    return this.call<string[]>("project_names");
  }

  taskDetails(taskId: number): Promise<TaskDetails> {
    return this.call<TaskDetails>("task_details", { taskId });
  }

  projectSummaries(): Promise<ProjectSummary[]> {
    return this.call<ProjectSummary[]>("project_summaries");
  }

  createProject(name: string): Promise<number> {
    return this.call<number>("create_project", { name });
  }

  async changeProject(projectId: number, change: ProjectChange): Promise<void> {
    await this.call("change_project", { projectId, change });
  }

  async removeProject(projectId: number): Promise<void> {
    await this.call("remove_project", { projectId });
  }

  projectColors(): Promise<string[]> {
    return this.call<string[]>("project_colors");
  }

  deliveredTasks(): Promise<DeliveredItem[]> {
    return this.call<DeliveredItem[]>("delivered_tasks");
  }

  async trashItem(taskId: number): Promise<void> {
    await this.call("trash_item", { taskId });
  }

  async restoreItem(taskId: number): Promise<void> {
    await this.call("restore_item", { taskId });
  }

  async deleteItemForever(taskId: number): Promise<void> {
    await this.call("delete_item_forever", { taskId });
  }

  async emptyTrash(): Promise<void> {
    await this.call("empty_trash");
  }

  trashedItems(): Promise<TrashedItem[]> {
    return this.call<TrashedItem[]>("trashed_items");
  }

  trashRetentionDays(): Promise<number> {
    return this.call<number>("trash_retention_days");
  }

  captureLessons(): Promise<CaptureLesson[]> {
    return this.call<CaptureLesson[]>("capture_lessons");
  }

  async recordCaptureLesson(lesson: CaptureLesson): Promise<void> {
    await this.call("record_capture_lesson", { lesson });
  }

  assistantContext(): Promise<AdvisorContext> {
    return this.call<AdvisorContext>("assistant_context");
  }

  async editTask(taskId: number, edit: TaskEdit): Promise<void> {
    await this.call("edit_task", { taskId, edit });
  }

  aiStatus(): Promise<AiProviderStatus[]> {
    return this.call<AiProviderStatus[]>("ai_status");
  }

  async saveAiKey(provider: AiProviderName, apiKey: string): Promise<void> {
    await this.call("save_ai_key", { provider, apiKey });
  }

  async removeAiKey(provider: AiProviderName): Promise<void> {
    await this.call("remove_ai_key", { provider });
  }

  aiFetch(request: ProxiedRequest): Promise<ProxiedResponse> {
    return this.call<ProxiedResponse>("ai_fetch", { request });
  }

  calendarStatus(): Promise<CalendarStatus> {
    return this.call<CalendarStatus>("calendar_status");
  }

  connectCalendar(provider: CalendarProvider): Promise<CalendarAccount> {
    return this.call<CalendarAccount>("connect_calendar", { provider });
  }

  async disconnectCalendar(accountId: number): Promise<void> {
    await this.call("disconnect_calendar", { accountId });
  }

  async syncCalendars(): Promise<void> {
    await this.call("sync_calendars");
  }

  async startRecording(mode: RecordingMode, video: VideoChoice | null): Promise<void> {
    await this.call("start_recording", { mode, video });
  }

  captureTargets(): Promise<CaptureTarget[]> {
    return this.call<CaptureTarget[]>("capture_targets");
  }

  async pauseRecording(): Promise<void> {
    await this.call("pause_recording");
  }

  async resumeRecording(): Promise<void> {
    await this.call("resume_recording");
  }

  async stopRecording(): Promise<void> {
    await this.call("stop_recording");
  }

  sessions(): Promise<SessionSummary[]> {
    return this.call<SessionSummary[]>("sessions");
  }

  sessionDetails(sessionId: number): Promise<SessionDetails> {
    return this.call<SessionDetails>("session_details", { sessionId });
  }

  searchSessions(text: string): Promise<SessionSearchHit[]> {
    return this.call<SessionSearchHit[]>("search_sessions", { text });
  }

  async deleteSession(sessionId: number): Promise<void> {
    await this.call("delete_session", { sessionId });
  }

  async retryTranscription(sessionId: number): Promise<void> {
    await this.call("retry_transcription", { sessionId });
  }

  async exportSession(sessionId: number): Promise<void> {
    await this.call("export_session", { sessionId });
  }

  sessionStorage(): Promise<SessionStorage> {
    return this.call<SessionStorage>("session_storage");
  }

  async saveSessionNotes(sessionId: number, notes: WrittenSessionNotes): Promise<void> {
    const { tasks, ...written } = notes;
    await this.call("save_session_notes", { sessionId, notes: written, tasks });
  }

  applySessionTask(actionItemId: number): Promise<number> {
    return this.call<number>("apply_session_task", { actionItemId });
  }

  onTranscriptReady(handler: (sessionId: number) => void): Promise<() => void> {
    return listen<number>(SHELL_EVENTS.transcriptReady, (event) => handler(event.payload));
  }

  mediaUrl(mediaPath: string): string {
    return convertFileSrc(mediaPath);
  }

  onModelDownload(handler: (progress: ModelDownload) => void): Promise<() => void> {
    return listen<ModelDownload>(SHELL_EVENTS.modelDownloadProgress, (event) => handler(event.payload));
  }

  async openMeetingLink(url: string): Promise<void> {
    await this.call("open_meeting_link", { url });
  }

  async openAdvancedSettings(section?: string): Promise<void> {
    await this.call("open_advanced_settings", { section: section ?? null });
  }

  autostartEnabled(): Promise<boolean> {
    return this.call<boolean>("autostart_enabled");
  }

  appVersion(): Promise<string> {
    return this.call<string>("app_version");
  }

  checkForUpdate(): Promise<AvailableUpdate | null> {
    return this.call<AvailableUpdate | null>("check_for_update");
  }

  async installUpdate(): Promise<void> {
    await this.call("install_update");
  }

  async setAutostart(enabled: boolean): Promise<void> {
    await this.call("set_autostart", { enabled });
  }

  async setHitArea(area: HitArea): Promise<void> {
    await this.call("set_island_hit_area", { area });
  }

  onAlert(handler: (alert: DueAlert) => void): Promise<() => void> {
    return listen<DueAlert>(SHELL_EVENTS.alertReady, (event) => handler(event.payload));
  }

  onSystemStats(handler: (stats: SystemStats) => void): Promise<() => void> {
    return listen<SystemStats>(SHELL_EVENTS.systemStats, (event) => handler(event.payload));
  }

  listen(eventName: string, handler: () => void): Promise<() => void> {
    return listen(eventName, handler);
  }

  /** Runs a command; a refusal from the core or the shell rejects with a `BackendError`. */
  private async call<T>(command: string, args?: InvokeArgs): Promise<T> {
    try {
      return await invoke<T>(command, args);
    } catch (failure) {
      throw BackendError.from(failure);
    }
  }
}
