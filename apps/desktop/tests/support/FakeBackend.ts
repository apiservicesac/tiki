import type { AdvisorContext, AiProviderName, InterpretedCapture, ProxiedRequest, ProxiedResponse, WrittenSessionNotes } from "../../src/ai";
import type {
  AiProviderStatus,
  AlertDelivery,
  AvailableUpdate,
  CalendarAccount,
  CaptureTarget,
  CalendarProvider,
  CalendarStatus,
  CaptureLesson,
  TikiAlert,
  ProjectChange,
  ProjectSummary,
  ModelDownload,
  RecordingMode,
  SessionDetails,
  SessionSearchHit,
  SessionStorage,
  SessionSummary,
  TranscriptSegment,
  DeliveredItem,
  DueAlert,
  HitArea,
  IslandSnapshot,
  TaskDetails,
  TaskEdit,
  TaskStatus,
  TikiBackend,
  TrashedItem,
  UserPreferences,
  VideoChoice,
  SystemStats,
} from "../../src/backend/contracts";
import { AI_PROVIDERS } from "../../src/backend/contracts";
import { BackendError } from "../../src/backend/BackendError";
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
    { projectId: 1, name: "Acme", color: "#2F6BD8", openCount: 2, pressingCount: 2, notes: "" },
    { projectId: 2, name: "Atlas", color: "#17837D", openCount: 3, pressingCount: 1, notes: "" },
    { projectId: 3, name: "Vertex", color: "#C2410C", openCount: 1, pressingCount: 0, notes: "" },
  ];
  delivered: DeliveredItem[] = [
    { taskId: 9, title: "Reporte de ventas de septiembre", projectId: 1, projectName: "Acme", projectColor: "#2F6BD8", tags: ["ventas"], deliveredLabel: "hoy" },
    { taskId: 8, title: "Ajuste del acta de inversiones", projectId: null, projectName: null, projectColor: null, tags: [], deliveredLabel: "ayer" },
  ];
  /** What the user sent to the trash, newest first. */
  trash: TrashedItem[] = [
    { taskId: 21, kind: "task", title: "Cotización de licencias antiguas", projectName: "Vertex", projectColor: "#C2410C", trashedLabel: "hoy", deletesInLabel: "se borra en 30 días" },
    { taskId: 22, kind: "reminder", title: "Llamar al banco", projectName: null, projectColor: null, trashedLabel: "hace 3 días", deletesInLabel: "se borra en 27 días" },
    { taskId: 23, kind: "task", title: "Borrador del acta de septiembre", projectName: "Acme", projectColor: "#2F6BD8", trashedLabel: "hace 29 días", deletesInLabel: "se borra mañana" },
  ];
  readonly restoredItems: number[] = [];
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
    transcriptionQuality: "balanced",
    sessionNotesAutomatic: true,
    sessionMediaDays: 90,
    showSystemStats: true,
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
  /** When set, saving a capture or an edit is refused with it, as the core would. */
  saveFailure: BackendError | null = null;
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
    if (this.saveFailure) {
      throw this.saveFailure;
    }
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

  async trashItem(taskId: number): Promise<void> {
    const item = this.snapshot.openItems.find((line) => line.taskId === taskId);
    if (item) {
      const { kind, title, projectName, projectColor } = item;
      this.trash = [{ taskId, kind, title, projectName, projectColor, trashedLabel: "hoy", deletesInLabel: "se borra en 30 días" }, ...this.trash];
    }
  }

  async restoreItem(taskId: number): Promise<void> {
    this.restoredItems.push(taskId);
    this.trash = this.trash.filter((item) => item.taskId !== taskId);
  }

  async deleteItemForever(taskId: number): Promise<void> {
    this.trash = this.trash.filter((item) => item.taskId !== taskId);
  }

  async emptyTrash(): Promise<void> {
    this.trash = [];
  }

  async trashedItems(): Promise<TrashedItem[]> {
    return this.trash;
  }

  async trashRetentionDays(): Promise<number> {
    return 30;
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
          projectName: "Acme",
          tags: [],
          status: "to_do",
          priority: "high",
          dueDate: "2026-10-22",
          remindAt: null,
          waitingDays: null,
        },
      ],
      projects: [
        { name: "Acme", notes: "Contacto: Lucía." },
        { name: "Atlas", notes: "" },
        { name: "Vertex", notes: "" },
      ],
    };
  }

  async editTask(taskId: number, edit: TaskEdit): Promise<void> {
    if (this.saveFailure) {
      throw this.saveFailure;
    }
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
    const account = { accountId: this.calendar.accounts.length + 1, provider, email: `ana@${provider}.test`, displayName: "Ana Pérez", avatar: null };
    this.calendar = { ...this.calendar, accounts: [...this.calendar.accounts, account] };
    return account;
  }

  async disconnectCalendar(accountId: number): Promise<void> {
    this.calendar = { ...this.calendar, accounts: this.calendar.accounts.filter((account) => account.accountId !== accountId) };
  }

  async syncCalendars(): Promise<void> {}

  /** What the island asked the recorder, in order: `start:audio_only`, `pause`, `stop`… */
  readonly recordingRequests: string[] = [];

  /** The screen or window the last video session records. */
  lastVideoChoice: VideoChoice | null = null;
  readonly screensAndWindows: CaptureTarget[] = [
    { kind: "screen", id: 1, name: "" },
    { kind: "window", id: 1_311_754, name: "Reunión de Teams" },
  ];

  async startRecording(mode: RecordingMode, video: VideoChoice | null): Promise<void> {
    this.recordingRequests.push(`start:${mode}`);
    this.lastVideoChoice = video;
    this.snapshot.recording = { sessionId: 1, mode, paused: false, recordedMs: 0 };
  }

  async captureTargets(): Promise<CaptureTarget[]> {
    return this.screensAndWindows;
  }

  async pauseRecording(): Promise<void> {
    this.recordingRequests.push("pause");
    this.changeRecording(true);
  }

  async resumeRecording(): Promise<void> {
    this.recordingRequests.push("resume");
    this.changeRecording(false);
  }

  async stopRecording(): Promise<void> {
    this.recordingRequests.push("stop");
    this.snapshot.recording = null;
  }

  /** A ready session with its transcript, and one whose transcription failed. */
  sessionList: SessionSummary[] = [
    { sessionId: 2, title: "Revisión con Atlas", mode: "audio_and_video", status: "ready", startedAt: "2026-10-21T10:00:00", startedLabel: "Mié 21 · 10:00", recordedMs: 1_830_000, hasMedia: true, mediaBytes: 21_000_000, meetingTitle: "Revisión con Atlas" },
    { sessionId: 1, title: "Sesión del Mar 20, 16:00", mode: "audio_only", status: "failed", startedAt: "2026-10-20T16:00:00", startedLabel: "Mar 20 · 16:00", recordedMs: 600_000, hasMedia: true, mediaBytes: 3_000_000, meetingTitle: null },
  ];
  readonly sessionTranscripts = new Map<number, TranscriptSegment[]>([
    [2, [
      { speaker: "others", startMs: 0, endMs: 4_000, text: "Revisemos la facturación de octubre" },
      { speaker: "you", startMs: 5_000, endMs: 8_000, text: "Mañana envío el informe" },
      { speaker: "others", startMs: 9_000, endMs: 12_000, text: "Perfecto, gracias" },
    ]],
  ]);
  readonly deletedSessions: number[] = [];
  /** Notes saved per session, and the proposed tasks applied, in order. */
  readonly savedSessionNotes = new Map<number, WrittenSessionNotes>();
  readonly appliedSessionTasks: number[] = [];
  private transcriptReadyHandler: ((sessionId: number) => void) | null = null;
  readonly retriedTranscriptions: number[] = [];
  private modelDownloadHandler: ((progress: ModelDownload) => void) | null = null;

  async sessions(): Promise<SessionSummary[]> {
    return this.sessionList;
  }

  async sessionDetails(sessionId: number): Promise<SessionDetails> {
    const summary = this.sessionList.find((session) => session.sessionId === sessionId);
    if (!summary) {
      throw new BackendError("not_found", "no such session");
    }
    return {
      summary,
      language: "es",
      transcript: this.sessionTranscripts.get(sessionId) ?? [],
      notes: this.savedSessionNotes.get(sessionId) ?? null,
      actionItems: (this.savedSessionNotes.get(sessionId)?.tasks ?? []).map((task, index) => {
        const actionItemId = index + 1;
        return {
          actionItemId,
          title: task.title,
          dueDate: task.dueDate,
          taskId: this.appliedSessionTasks.includes(actionItemId) ? FakeBackend.taskOf(actionItemId) : null,
        };
      }),
      mediaPath: summary.hasMedia ? `C:\\Tiki\\sessions\\${sessionId}\\session.mp4` : null,
    };
  }

  async searchSessions(text: string): Promise<SessionSearchHit[]> {
    const word = text.trim().toLowerCase();
    return [...this.sessionTranscripts].flatMap(([sessionId, lines]) => lines
      .filter((line) => line.text.toLowerCase().includes(word))
      .map((line) => ({ sessionId, sessionTitle: "Revisión con Atlas", startedAt: "2026-10-21T10:00:00", speaker: line.speaker, startMs: line.startMs, text: line.text })));
  }

  async deleteSession(sessionId: number): Promise<void> {
    this.deletedSessions.push(sessionId);
    this.sessionList = this.sessionList.filter((session) => session.sessionId !== sessionId);
  }

  async retryTranscription(sessionId: number): Promise<void> {
    this.retriedTranscriptions.push(sessionId);
  }

  readonly exportedSessions: number[] = [];

  async exportSession(sessionId: number): Promise<void> {
    this.exportedSessions.push(sessionId);
  }

  async sessionStorage(): Promise<SessionStorage> {
    return { mediaBytes: 24_000_000, mediaDays: 90 };
  }

  async saveSessionNotes(sessionId: number, notes: WrittenSessionNotes): Promise<void> {
    this.savedSessionNotes.set(sessionId, notes);
  }

  async applySessionTask(actionItemId: number): Promise<number> {
    this.appliedSessionTasks.push(actionItemId);
    return FakeBackend.taskOf(actionItemId);
  }

  /** The task a proposed task becomes, numbered after the sample tasks. */
  private static taskOf(actionItemId: number): number {
    const firstFreeTaskId = 100;
    return firstFreeTaskId + actionItemId;
  }

  async onTranscriptReady(handler: (sessionId: number) => void): Promise<() => void> {
    this.transcriptReadyHandler = handler;
    return () => {
      this.transcriptReadyHandler = null;
    };
  }

  /** Simulates the shell saying a session's transcript is ready. */
  reportTranscriptReady(sessionId: number): void {
    this.transcriptReadyHandler?.(sessionId);
  }

  mediaUrl(mediaPath: string): string {
    return `http://asset.localhost/${encodeURIComponent(mediaPath)}`;
  }

  async onModelDownload(handler: (progress: ModelDownload) => void): Promise<() => void> {
    this.modelDownloadHandler = handler;
    return () => {
      this.modelDownloadHandler = null;
    };
  }

  /** Simulates the shell reporting the model download. */
  reportModelDownload(progress: ModelDownload): void {
    this.modelDownloadHandler?.(progress);
  }

  private changeRecording(paused: boolean): void {
    if (this.snapshot.recording) {
      this.snapshot.recording = { ...this.snapshot.recording, paused };
    }
  }

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

  private systemStatsHandler: ((stats: SystemStats) => void) | null = null;

  /** Plays the shell measuring the computer. */
  deliverSystemStats(stats: SystemStats): void {
    this.systemStatsHandler?.(stats);
  }

  async onSystemStats(handler: (stats: SystemStats) => void): Promise<() => void> {
    this.systemStatsHandler = handler;
    return () => {
      this.systemStatsHandler = null;
    };
  }

  async onAlert(handler: (alert: DueAlert) => void): Promise<() => void> {
    this.alertHandler = handler;
    return () => {
      this.alertHandler = null;
    };
  }

  /** Simulates the shell delivering an alert; unless told otherwise, it may chime. */
  deliverAlert(alert: TikiAlert, delivery: AlertDelivery = FakeBackend.AUDIBLE): void {
    this.alertHandler?.({ ...alert, delivery });
  }

  /** How an alert arrives during the day, with every way of reaching the user on. */
  static readonly AUDIBLE: AlertDelivery = Object.freeze({ sound: true, windowsNotification: true });

  /** Simulates the desktop shell emitting an event. */
  emit(eventName: string): void {
    this.listeners.get(eventName)?.();
  }

  /** A sample day of pending work on Wednesday, October 21, 2026. */
  static sampleSnapshot(): IslandSnapshot {
    const acme = { projectId: 1, projectName: "Acme", projectColor: "#2F6BD8" };
    const atlas = { projectId: 2, projectName: "Atlas", projectColor: "#17837D" };
    const vertex = { projectId: 3, projectName: "Vertex", projectColor: "#C2410C" };
    const access = { taskId: 3, kind: "task" as const, title: "Acceso solo a análisis de facturas de Contoso", ...atlas, detailLabel: "1 día de atraso", urgency: "critical" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const tracking = { taskId: 1, kind: "task" as const, title: "Tracking de formularios comerciales", ...acme, detailLabel: "mañana", urgency: "soon" as const, priority: "high" as const, tags: [], at: null, repeat: "never" as const };
    const sales = { taskId: 2, kind: "task" as const, title: "Mejora en el módulo de Ventas", ...acme, detailLabel: "Vie 23", urgency: "soon" as const, priority: "normal" as const, tags: ["ventas"], at: "2026-10-24T16:00:00", repeat: "never" as const };
    const vacation = { taskId: 4, kind: "task" as const, title: "Saldo vacacional con solicitud rechazada", ...atlas, detailLabel: "En curso", urgency: "normal" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const github = { taskId: 5, kind: "task" as const, title: "Permisos de GitHub para impuestos", ...vertex, detailLabel: "esperando 3 días", urgency: "waiting" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    const water = { taskId: 8, kind: "reminder" as const, title: "Tomar agua", projectId: null, projectName: null, projectColor: null, detailLabel: "Vie 23 · 11:00 · Cada día", urgency: "reminder" as const, priority: "normal" as const, tags: [], at: "2026-10-23T11:00:00", repeat: "daily" as const };
    const zones = { taskId: 6, kind: "task" as const, title: "Confirmación de zonas del dashboard de cobranzas", ...atlas, detailLabel: "esperando 1 día", urgency: "waiting" as const, priority: "normal" as const, tags: [], at: null, repeat: "never" as const };
    return {
      mood: "worried",
      moodLabel: "ATRASO",
      inCall: false,
      recording: null,
      attentionCount: 5,
      dueCount: 3,
      waitingCount: 2,
      openCount: 6,
      deliveredToday: 1,
      closedSummary: "3 por atender · 2 esperando",
      todaySubtitle: "miércoles 21 de octubre · 3 por atender",
      narration: "Buenos días. Hoy es miércoles 21 de octubre.",
      sections: [
        { key: "overdue", title: "Atrasado", items: [access] },
        { key: "due_soon", title: "Vence pronto", items: [tracking, sales] },
        { key: "follow_up", title: "Toca insistir", items: [github] },
      ],
      openItems: [access, tracking, sales, vacation, github, zones, water],
      focus: { item: access, reason: "Está atrasada" },
      agenda: [
        {
          day: "2026-10-23",
          label: "Hoy",
          entries: [
            {
              kind: "meeting",
              timeLabel: "09:00–09:30",
              meeting: {
                title: "Daily de Acme",
                startsAt: "2026-10-23T09:00:00",
                endsAt: "2026-10-23T09:30:00",
                allDay: false,
                location: "Teams",
                joinUrl: "https://teams.microsoft.com/l/meetup-join/daily",
                platform: "teams",
                account: { provider: "microsoft", email: "ana@microsoft.test", displayName: "Ana Pérez", avatar: null },
              },
            },
            { kind: "reminder", timeLabel: "11:00", item: water },
          ],
        },
        { day: "2026-10-24", label: "Mañana", entries: [{ kind: "task", timeLabel: "16:00", item: sales }] },
      ],
    };
  }
}
