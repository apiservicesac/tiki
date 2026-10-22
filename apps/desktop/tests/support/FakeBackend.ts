import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../../src/ai";
import type {
  AiProviderStatus,
  AvailableUpdate,
  CaptureLesson,
  TikiAlert,
  ProjectChange,
  ProjectSummary,
  DeliveredItem,
  HitArea,
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
  readonly movedTasks: Array<{ taskId: number; status: TaskStatus }> = [];
  readonly hitAreas: HitArea[] = [];
  readonly confirmedCaptures: InterpretedCapture[] = [];
  readonly editedTasks: Array<{ taskId: number; edit: TaskEdit }> = [];
  projects: ProjectSummary[] = [
    { projectId: 1, name: "Altamira", color: "#2F6BD8", openCount: 2, pressingCount: 2, notes: "" },
    { projectId: 2, name: "Andeanvet", color: "#17837D", openCount: 3, pressingCount: 1, notes: "" },
    { projectId: 3, name: "Vapestation", color: "#C2410C", openCount: 1, pressingCount: 0, notes: "" },
  ];
  delivered: DeliveredItem[] = [
    { taskId: 9, title: "Reporte de ventas de septiembre", projectName: "Altamira", projectColor: "#2F6BD8", deliveredLabel: "hoy" },
    { taskId: 8, title: "Ajuste del acta CAPEX", projectName: null, projectColor: null, deliveredLabel: "ayer" },
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

  async projectNames(): Promise<string[]> {
    return this.projects.map((project) => project.name);
  }

  async projectSummaries(): Promise<ProjectSummary[]> {
    return this.projects;
  }

  async createProject(name: string): Promise<number> {
    const projectId = Math.max(0, ...this.projects.map((project) => project.projectId)) + 1;
    this.projects = [...this.projects, { projectId, name, color: "#7357E8", openCount: 0, pressingCount: 0, notes: "" }];
    return projectId;
  }

  async changeProject(projectId: number, change: ProjectChange): Promise<void> {
    this.projects = this.projects.map((project) => (project.projectId === projectId ? { ...project, ...change } : project));
  }

  async removeProject(projectId: number): Promise<void> {
    this.projects = this.projects.filter((project) => project.projectId !== projectId);
  }

  async projectColors(): Promise<string[]> {
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
      projectName: item?.projectName ?? null,
      dueDate: "2026-10-22",
      priority: "normal",
      status: "to_do",
      remindAt: null,
      kind: "task",
      tags: [],
    };
  }

  readonly lessons: CaptureLesson[] = [];

  async captureLessons(): Promise<CaptureLesson[]> {
    return this.lessons;
  }

  async recordCaptureLesson(lesson: CaptureLesson): Promise<void> {
    this.lessons.unshift(lesson);
  }

  async assistantContext(): Promise<AdvisorContext> {
    return {
      now: "2026-10-21T17:30:00",
      openTasks: [
        {
          taskId: 1,
          kind: "task",
          title: "Tracking de formularios",
          projectName: "Altamira",
          tags: [],
          status: "to_do",
          priority: "high",
          dueDate: "2026-10-22",
          remindAt: null,
          waitingDays: null,
        },
      ],
      projects: [{ name: "Altamira", notes: "Contacto: Lucía." }],
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
    const altamira = { projectName: "Altamira", projectColor: "#2F6BD8" };
    const andeanvet = { projectName: "Andeanvet", projectColor: "#17837D" };
    const vapestation = { projectName: "Vapestation", projectColor: "#C2410C" };
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
            { taskId: 3, title: "Acceso solo a análisis de facturas de Almendra", ...andeanvet, detailLabel: "1 día de atraso", urgency: "critical", tags: [] },
          ],
        },
        {
          key: "due_soon",
          title: "Vence pronto",
          items: [
            { taskId: 1, title: "Tracking de formularios comerciales", ...altamira, detailLabel: "mañana", urgency: "soon", tags: [] },
            { taskId: 2, title: "Mejora en el módulo de Ventas", ...altamira, detailLabel: "Vie 23", urgency: "soon", tags: [] },
          ],
        },
        {
          key: "your_turn",
          title: "Te toca",
          items: [
            { taskId: 4, title: "Saldo vacacional con solicitud rechazada", ...andeanvet, detailLabel: "En prueba", urgency: "normal", tags: [] },
          ],
        },
        {
          key: "waiting",
          title: "Esperando a otro",
          items: [
            { taskId: 5, title: "Permisos de GitHub para retenciones de IGV", ...vapestation, detailLabel: "3 días", urgency: "waiting", tags: [] },
            { taskId: 6, title: "Confirmación de zonas del dashboard CxC", ...andeanvet, detailLabel: "1 día", urgency: "waiting", tags: [] },
          ],
        },
      ],
    };
  }
}
