import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../../src/ai";
import type {
  AiProviderStatus,
  AvailableUpdate,
  TikiAlert,
  CaptureIntent,
  ClientChange,
  ClientSummary,
  DeliveredItem,
  HitArea,
  CaptureOutcome,
  IslandSnapshot,
  TaskDetails,
  TaskEdit,
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
  readonly editedTasks: Array<{ taskId: number; edit: TaskEdit }> = [];
  clients: ClientSummary[] = [
    { clientId: 1, name: "Altamira", color: "#2F6BD8", openCount: 2, pressingCount: 2 },
    { clientId: 2, name: "Andeanvet", color: "#17837D", openCount: 3, pressingCount: 1 },
    { clientId: 3, name: "Vapestation", color: "#C2410C", openCount: 1, pressingCount: 0 },
  ];
  delivered: DeliveredItem[] = [
    { taskId: 9, title: "Reporte de ventas de septiembre", clientName: "Altamira", clientColor: "#2F6BD8", deliveredLabel: "hoy" },
    { taskId: 8, title: "Ajuste del acta CAPEX", clientName: null, clientColor: null, deliveredLabel: "ayer" },
  ];
  savedPreferences: UserPreferences = {
    locale: "spanish",
    islandPlacement: "top",
    aiProvider: "anthropic",
    aiModel: null,
    morningBriefTime: "08:30:00",
    eveningBriefTime: "18:00:00",
    notificationsEnabled: true,
    alertSound: true,
    windowsNotifications: true,
    focusMode: false,
  };
  autostart = false;
  availableUpdate: AvailableUpdate | null = null;
  installedUpdates = 0;
  readonly openedSettings: Array<string | undefined> = [];
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
      : { kind: "task", title: trimmed, clientId: null, clientName: null, dueDate: null, remindAt: null };
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
    return this.clients.map((client) => client.name);
  }

  async clientSummaries(): Promise<ClientSummary[]> {
    return this.clients;
  }

  async createClient(name: string): Promise<number> {
    const clientId = Math.max(0, ...this.clients.map((client) => client.clientId)) + 1;
    this.clients = [...this.clients, { clientId, name, color: "#7357E8", openCount: 0, pressingCount: 0 }];
    return clientId;
  }

  async changeClient(clientId: number, change: ClientChange): Promise<void> {
    this.clients = this.clients.map((client) => (client.clientId === clientId ? { ...client, ...change } : client));
  }

  async removeClient(clientId: number): Promise<void> {
    this.clients = this.clients.filter((client) => client.clientId !== clientId);
  }

  async clientColors(): Promise<string[]> {
    return ["#2F6BD8", "#17837D", "#C2410C", "#7357E8"];
  }

  async deliveredTasks(): Promise<DeliveredItem[]> {
    return this.delivered;
  }

  async taskDetails(taskId: number): Promise<TaskDetails> {
    const item = this.snapshot.sections.flatMap((section) => section.items).find((line) => line.taskId === taskId);
    return {
      taskId,
      title: item?.title ?? "",
      notes: "",
      clientName: item?.clientName ?? null,
      dueDate: "2026-10-22",
      priority: "normal",
      status: "to_do",
      remindAt: null,
    };
  }

  async assistantContext(): Promise<AdvisorContext> {
    return {
      today: "2026-10-21",
      openTasks: [
        { title: "Tracking de formularios", clientName: "Altamira", status: "to_do", priority: "high", dueDate: "2026-10-22", waitingDays: null },
      ],
    };
  }

  async editTask(taskId: number, edit: TaskEdit): Promise<void> {
    this.editedTasks.push({ taskId, edit });
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

  async autostartEnabled(): Promise<boolean> {
    return this.autostart;
  }

  async openAdvancedSettings(section?: string): Promise<void> {
    this.openedSettings.push(section);
  }

  async appVersion(): Promise<string> {
    return "0.2.0";
  }

  async checkForUpdate(): Promise<AvailableUpdate | null> {
    return this.availableUpdate;
  }

  async installUpdate(): Promise<void> {
    this.installedUpdates += 1;
  }

  async setAutostart(enabled: boolean): Promise<void> {
    this.autostart = enabled;
  }

  async setHitArea(area: HitArea): Promise<void> {
    this.hitAreas.push(area);
  }

  async listen(eventName: string, handler: () => void): Promise<() => void> {
    this.listeners.set(eventName, handler);
    return () => this.listeners.delete(eventName);
  }

  private alertHandler: ((brief: TikiAlert) => void) | null = null;

  async onAlert(handler: (brief: TikiAlert) => void): Promise<() => void> {
    this.alertHandler = handler;
    return () => {
      this.alertHandler = null;
    };
  }

  /** Simulates the shell delivering a brief. */
  deliverAlert(brief: TikiAlert): void {
    this.alertHandler?.(brief);
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
      deliveredToday: 1,
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
