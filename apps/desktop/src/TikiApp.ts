import { TikiAi } from "./ai";
import { TikiAssistant } from "./assistant/TikiAssistant";
import { LOCALE_CODES, type TikiBackend } from "./backend/contracts";
import { TauriBackend } from "./backend/TauriBackend";
import { CaptureView } from "./capture/CaptureView";
import { Translator } from "./i18n/Translator";
import { IslandController } from "./island/IslandController";
import { IslandView } from "./island/IslandView";
import { HitAreaReporter } from "./island/HitAreaReporter";
import { MascotRenderer } from "./mascot/MascotRenderer";

/** Wires the backend, the translator and the island together. */
export class TikiApp {
  private readonly hitAreaReporter: HitAreaReporter;

  constructor(
    private readonly root: HTMLElement,
    private readonly backend: TikiBackend = new TauriBackend(),
  ) {
    this.hitAreaReporter = new HitAreaReporter(backend);
  }

  async start(): Promise<IslandController> {
    const [preferences, snapshot] = await Promise.all([
      this.backend.preferences(),
      this.backend.islandSnapshot(),
    ]);
    const translator = new Translator(LOCALE_CODES[preferences.locale]);

    const captureView = new CaptureView(translator);
    const view = new IslandView(this.root, translator, new MascotRenderer(), captureView);
    const controller = new IslandController(
      this.backend,
      view,
      captureView,
      this.hitAreaReporter,
      snapshot,
      preferences,
      translator,
      new TikiAssistant(this.backend, new TikiAi((request) => this.backend.aiFetch(request))),
    );
    await controller.start();
    return controller;
  }
}
