import type { AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../../src/ai";
import type {
  AiProviderStatus,
  CaptureIntent,
  HitArea,
  CaptureOutcome,
  IslandSnapshot,
  TaskStatus,
  TikiBackend,
  UserPreferences,
} from "../../src/backend/contracts";
import { AI_PROVIDERS } from "../../src/backend/contracts";
import { FakeShell } from "./FakeShell";

/**
 * Test double for the Rust core: serves a fixed sample day, records what the interface
 * asked for and lets tests fire shell events.
 */
export class FakeBackend implements TikiBackend {
  readonly capturedSentences: string[] = [];
  readonly movedTasks: Array<{ taskId: number; status: TaskStatus }> = [];
  readonly hitAreas: HitArea[] = [];
  readonly confirmedCaptures: InterpretedCapture[] = [];
  savedPreferences: UserPreferences = { locale: "spanish", islandPlacement: "top", aiProvider: "anthropic", aiModel: null };
  readonly aiKeys = new Map<AiProviderName, string>();
  /** Plays the AI provider: what the official SDK receives back through the shell. */
  aiShell = new FakeShell(() => ({}));
  private readonly listeners = new Map<string, () => void>();

  constructor(private readonly snapshot: IslandSnapshot = FakeBackend.sampleSnapshot()) {}

  async islandSnapshot(): Promise<IslandSnapshot> {
    return this.snapshot;
  }

  async previewCapture(sentence: string): Promise<CaptureIntent> {
    const trimmed = sentence.trim();
    return trimmed.startsWith("+")
      ? { kind: "client", name: trimmed.slice(1).trim() }
      : { kind: "task", title: trimmed, clientId: null, clientName: null, dueDate: null };
  }

  async capture(sentence: string): Promise<CaptureOutcome> {
    this.capturedSentences.push(sentence.trim());
    return { kind: "task", id: this.capturedSentences.length };
  }

  async moveTask(taskId: number, status: TaskStatus): Promise<void> {
    this.movedTasks.push({ taskId, status });
  }

  async preferences(): Promise<UserPreferences> {
    return this.savedPreferences;
  }

  async updatePreferences(preferences: UserPreferences): Promise<void> {
    this.savedPreferences = preferences;
  }

  async saveConfirmedCapture(capture: InterpretedCapture): Promise<void> {
    this.confirmedCaptures.push(capture);
  }

  async clientNames(): Promise<string[]> {
    return ["Altamira", "Andeanvet", "Vapestation"];
  }

  async aiStatus(): Promise<AiProviderStatus[]> {
    return AI_PROVIDERS.map((provider) => ({ provider, configured: this.aiKeys.has(provider) }));
  }

  async saveAiKey(provider: AiProviderName, apiKey: string): Promise<void> {
    this.aiKeys.set(provider, apiKey);
  }

  async removeAiKey(provider: AiProviderName): Promise<void> {
    this.aiKeys.delete(provider);
  }

  aiFetch(request: ProxiedRequest): Promise<ProxiedResponse> {
    return this.aiShell.send(request);
  }

  async setHitArea(area: HitArea): Promise<void> {
    this.hitAreas.push(area);
  }

  async listen(eventName: string, handler: () => void): Promise<() => void> {
    this.listeners.set(eventName, handler);
    return () => this.listeners.delete(eventName);
  }

  /** Simulates the desktop shell emitting an event. */
  emit(eventName: string): void {
    this.listeners.get(eventName)?.();
  }

  /** The user's real pending work on Wednesday, October 21, 2026. */
  static sampleSnapshot(): IslandSnapshot {
    const altamira = { clientName: "Altamira", clientColor: "#2F6BD8" };
    const andeanvet = { clientName: "Andeanvet", clientColor: "#17837D" };
    const vapestation = { clientName: "Vapestation", clientColor: "#C2410C" };
    return {
      mood: "worried",
      moodLabel: "ATRASO",
      attentionCount: 5,
      dueCount: 3,
      waitingCount: 2,
      closedSummary: "3 hoy · 2 esperando",
      todaySubtitle: "miércoles 21 de octubre · 3 vencen pronto",
      narration: "Buenos días. Hoy es miércoles 21 de octubre.",
      sections: [
        {
          key: "overdue",
          title: "Atrasado",
          items: [
            { taskId: 3, title: "Acceso solo a análisis de facturas de Almendra", ...andeanvet, detailLabel: "1 día de atraso", urgency: "critical" },
          ],
        },
        {
          key: "due_soon",
          title: "Vence pronto",
          items: [
            { taskId: 1, title: "Tracking de formularios comerciales", ...altamira, detailLabel: "mañana", urgency: "soon" },
            { taskId: 2, title: "Mejora en el módulo de Ventas", ...altamira, detailLabel: "Vie 23", urgency: "soon" },
          ],
        },
        {
          key: "your_turn",
          title: "Te toca",
          items: [
            { taskId: 4, title: "Saldo vacacional con solicitud rechazada", ...andeanvet, detailLabel: "En prueba", urgency: "normal" },
          ],
        },
        {
          key: "waiting",
          title: "Esperando a otro",
          items: [
            { taskId: 5, title: "Permisos de GitHub para retenciones de IGV", ...vapestation, detailLabel: "3 días", urgency: "waiting" },
            { taskId: 6, title: "Confirmación de zonas del dashboard CxC", ...andeanvet, detailLabel: "1 día", urgency: "waiting" },
          ],
        },
      ],
    };
  }
}
