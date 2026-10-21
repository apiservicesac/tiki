import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SHELL_EVENTS } from "../config/uiConfig";
import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../ai";
import type {
  AiProviderStatus,
  AvailableUpdate,
  ClientChange,
  ClientSummary,
  DeliveredItem,
  TikiAlert,
  CaptureIntent,
  HitArea,
  CaptureOutcome,
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

  previewCapture(sentence: string): Promise<CaptureIntent> {
    return invoke<CaptureIntent>("preview_capture", { sentence });
  }

  capture(sentence: string): Promise<CaptureOutcome> {
    return invoke<CaptureOutcome>("capture", { sentence });
  }

  async moveTask(taskId: number, status: TaskStatus): Promise<void> {
    await invoke("move_task", { taskId, status });
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

  clientNames(): Promise<string[]> {
    return invoke<string[]>("client_names");
  }

  taskDetails(taskId: number): Promise<TaskDetails> {
    return invoke<TaskDetails>("task_details", { taskId });
  }

  clientSummaries(): Promise<ClientSummary[]> {
    return invoke<ClientSummary[]>("client_summaries");
  }

  createClient(name: string): Promise<number> {
    return invoke<number>("create_client", { name });
  }

  async changeClient(clientId: number, change: ClientChange): Promise<void> {
    await invoke("change_client", { clientId, change });
  }

  async removeClient(clientId: number): Promise<void> {
    await invoke("remove_client", { clientId });
  }

  clientColors(): Promise<string[]> {
    return invoke<string[]>("client_colors");
  }

  deliveredTasks(): Promise<DeliveredItem[]> {
    return invoke<DeliveredItem[]>("delivered_tasks");
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

  onAlert(handler: (brief: TikiAlert) => void): Promise<() => void> {
    return listen<TikiAlert>(SHELL_EVENTS.briefReady, (event) => handler(event.payload));
  }

  listen(eventName: string, handler: () => void): Promise<() => void> {
    return listen(eventName, handler);
  }
}
