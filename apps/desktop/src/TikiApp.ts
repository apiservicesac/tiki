import { LOCALE_CODES, type TikiBackend } from "./backend/contracts";
import { TauriBackend } from "./backend/TauriBackend";
import { CaptureView } from "./capture/CaptureView";
import { Translator } from "./i18n/Translator";
import { IslandController, type LocaleChangedListener } from "./island/IslandController";
import { IslandView } from "./island/IslandView";
import { WindowFitter } from "./island/WindowFitter";
import { MascotRenderer } from "./mascot/MascotRenderer";

/** Wires the backend, the translator and the island together. */
export class TikiApp {
  private readonly windowFitter: WindowFitter;

  constructor(
    private readonly root: HTMLElement,
    private readonly backend: TikiBackend = new TauriBackend(),
    private readonly onLocaleChanged: LocaleChangedListener = () => window.location.reload(),
  ) {
    this.windowFitter = new WindowFitter(backend);
  }

  async start(): Promise<IslandController> {
    const [preferences, snapshot] = await Promise.all([
      this.backend.preferences(),
      this.backend.islandSnapshot(),
    ]);
    const translator = new Translator(LOCALE_CODES[preferences.locale]);
    document.documentElement.lang = translator.localeCode;

    const captureView = new CaptureView(translator);
    const view = new IslandView(this.root, translator, new MascotRenderer(), captureView);
    const controller = new IslandController(
      this.backend,
      view,
      captureView,
      this.windowFitter,
      snapshot,
      preferences,
      this.onLocaleChanged,
    );
    await controller.start();
    return controller;
  }
}
