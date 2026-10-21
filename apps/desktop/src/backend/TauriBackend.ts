import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../ai";
import type {
  AiProviderStatus,
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

  async setHitArea(area: HitArea): Promise<void> {
    await invoke("set_island_hit_area", { area });
  }

  listen(eventName: string, handler: () => void): Promise<() => void> {
    return listen(eventName, handler);
  }
}
