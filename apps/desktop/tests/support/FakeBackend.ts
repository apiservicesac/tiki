import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse } from "../../src/ai";
import type {
  AiProviderStatus,
  AlertDelivery,
  AvailableUpdate,
  CalendarAccount,
  CalendarProvider,
  CalendarStatus,
  CaptureLesson,
  TikiAlert,
  ProjectChange,
  ProjectSummary,
  DeliveredItem,
  DueAlert,
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
    quietHours: { enabled: true, from: "22:00:00", until: "07:00:00" },
  };
  autostart = false;
  availableUpdate: AvailableUpdate | null = null;
  installedUpdates = 0;
  readonly openedSettings: Array<string | undefined> = [];
  readonly aiKeys = new Map<AiProviderName, string>();
  /** Plays the AI provider: what the official SDK receives back through the shell. */
  aiShell = new FakeShell(() => ({}));
  calendar: CalendarStatus = { accounts: [], available: ["microsoft", "google"] };
  readonly openedLinks: string[] = [];
  private readonly listeners = new Map<string, () => void>();

  constructor(private readonly snapshot: IslandSnapshot = FakeBackend.sampleSnapshot()) {}

  async islandSnapshot(): Promise<IslandSnapshot> {
    return this.snapshot;
  }

  async moveTask(taskId: number, status: TaskStatus): Promise<void> {
    this.movedTasks.push({ taskId, status });
  }

  readonly snoozed: Array<{ taskId: number; minutes: number }> = [];

  async snooze(taskId: number, minutes: number): Promise<void> {
    this.snoozed.push({ taskId, minutes });
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
    const item = this.snapshot.openItems.find((line) => line.taskId === taskId);
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
      repeat: "never",
      waitingFor: "",
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
      projects: [
        { name: "Altamira", notes: "Contacto: Lucía." },
        { name: "Andeanvet", notes: "" },
        { name: "Vapestation", notes: "" },
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

  async calendarStatus(): Promise<CalendarStatus> {
    return this.calendar;
  }

  async connectCalendar(provider: CalendarProvider): Promise<CalendarAccount> {
    const account = { accountId: this.calendar.accounts.length + 1, provider, email: `ana@${provider}.test` };
    this.calendar = { ...this.calendar, accounts: [...this.calendar.accounts, account] };
    return account;
  }

  async disconnectCalendar(accountId: number): Promise<void> {
    this.calendar = { ...this.calendar, accounts: this.calendar.accounts.filter((account) => account.accountId !== accountId) };
  }

  async syncCalendars(): Promise<void> {}

  async openMeetingLink(url: string): Promise<void> {
    this.openedLinks.push(url);
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

  private alertHandler: ((alert: DueAlert) => void) | null = null;

  async onAlert(handler: (alert: DueAlert) => void): Promise<() => void> {
    this.alertHandler = handler;
    return () => {
      this.alertHandler = null;
    };
  }

  /** Simulates the shell delivering a brief; unless told otherwise, it may chime. */
  deliverAlert(brief: TikiAlert, delivery: AlertDelivery = FakeBackend.AUDIBLE): void {
    this.alertHandler?.({ ...brief, delivery });
  }

  /** How an alert arrives during the day, with every way of reaching the user on. */
  static readonly AUDIBLE: AlertDelivery = Object.freeze({ sound: true, windowsNotification: true });

  /** Simulates the desktop shell emitting an event. */
  emit(eventName: string): void {
    this.listeners.get(eventName)?.();
  }

  /** The user's real pending work on Wednesday, October 21, 2026. */
  static sampleSnapshot(): IslandSnapshot {
    const altamira = { projectName: "Altamira", projectColor: "#2F6BD8" };
    const andeanvet = { projectName: "Andeanvet", projectColor: "#17837D" };
    const vapestation = { projectName: "Vapestation", projectColor: "#C2410C" };
    const access = { taskId: 3, title: "Acceso solo a análisis de facturas de Almendra", ...andeanvet, detailLabel: "1 día de atraso", urgency: "critical" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const tracking = { taskId: 1, title: "Tracking de formularios comerciales", ...altamira, detailLabel: "mañana", urgency: "soon" as const, priority: "high" as const, tags: [], at: null, repeat: "never" as const };
    const sales = { taskId: 2, title: "Mejora en el módulo de Ventas", ...altamira, detailLabel: "Vie 23", urgency: "soon" as const, priority: "normal" as const, tags: ["ventas"], at: null, repeat: "never" as const };
    const vacation = { taskId: 4, title: "Saldo vacacional con solicitud rechazada", ...andeanvet, detailLabel: "En curso", urgency: "normal" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const github = { taskId: 5, title: "Permisos de GitHub para retenciones de IGV", ...vapestation, detailLabel: "3 días", urgency: "waiting" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const water = { taskId: 8, title: "Tomar agua", projectName: null, projectColor: null, detailLabel: "Vie 23 · 11:00 · Cada día", urgency: "reminder" as const, priority: "normal" as const, tags: [], at: "2026-10-23T11:00:00", repeat: "daily" as const };
    const zones = { taskId: 6, title: "Confirmación de zonas del dashboard CxC", ...andeanvet, detailLabel: "1 día", urgency: "waiting" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
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
        { key: "overdue", title: "Atrasado", items: [access] },
        { key: "due_soon", title: "Vence pronto", items: [tracking, sales] },
        { key: "follow_up", title: "Toca insistir", items: [github] },
      ],
      openItems: [access, tracking, sales, vacation, github, zones, water],
      focus: { item: access, reason: "Está atrasada" },
      meetings: [
        {
          title: "Daily de Altamira",
          startsAt: "2026-10-23T09:00:00",
          endsAt: "2026-10-23T09:30:00",
          allDay: false,
          location: "Teams",
          joinUrl: "https://teams.microsoft.com/l/meetup-join/daily",
          accountEmail: "ana@microsoft.test",
        },
      ],
    };
  }
}
