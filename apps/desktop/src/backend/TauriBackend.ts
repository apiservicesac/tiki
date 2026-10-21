import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CaptureIntent,
  HitArea,
  CaptureOutcome,
  IslandSnapshot,
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

  async setHitArea(area: HitArea): Promise<void> {
    await invoke("set_island_hit_area", { area });
  }

  listen(eventName: string, handler: () => void): Promise<() => void> {
    return listen(eventName, handler);
  }
}
