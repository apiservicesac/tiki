import type { ActiveRecording } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import type { IslandPanel, IslandTab } from "./islandState";
import type { RecordingIndicator } from "./RecordingIndicator";

/** A tab as the header shows it. */
export interface IslandTabLabel {
  tab: IslandTab;
  labelKey: string;
}

/** The panels reached with a back button, each titled in the header. */
type TitledPanel = Exclude<IslandPanel, "tabs">;

export interface HeaderHandlers {
  onSelectTab: (tab: IslandTab) => void;
  onOpenPanel: (panel: IslandPanel) => void;
  onClose: () => void;
  onPauseRecording: () => void;
  onResumeRecording: () => void;
  onStopRecording: () => void;
}

/**
 * Top row of the expanded island: Tiki's face, then the tabs (or a back button and the
 * panel's title), and on the right the actions that make sense there. On the tabs: new and
 * settings; inside a panel only close, since the back button already leads out.
 * While a session records, its red dot and timer stay in the header everywhere, with pause
 * and stop on the tabs.
 */
export class HeaderView {
  private static readonly PANEL_TITLES: Readonly<Record<TitledPanel, string>> = Object.freeze({
    capture: "island.action.new",
    settings: "settings.title",
    edit: "edit.title",
    projects: "projects.title",
    brief: "receipt.title",
    record: "record.title",
    session: "session.title",
  });

  constructor(
    private readonly translator: Translator,
    private readonly recordingIndicator: RecordingIndicator,
  ) {}

  render(
    face: HTMLElement,
    panel: IslandPanel,
    tabs: readonly IslandTabLabel[],
    activeTab: IslandTab,
    recording: ActiveRecording | null,
    handlers: HeaderHandlers,
  ): HTMLElement {
    const leading = panel === "tabs"
      ? this.tabs(tabs, activeTab, handlers.onSelectTab)
      : this.backButton(panel, handlers.onOpenPanel);
    const close = ElementFactory.iconButton(ICONS.close, this.translator.text("island.action.close"), handlers.onClose);
    const actions = panel === "tabs"
      ? [
        ...this.recordingControls(recording, handlers),
        ElementFactory.iconButton(ICONS.plus, this.translator.text("island.action.new"), () =>
          handlers.onOpenPanel("capture")),
        ElementFactory.iconButton(ICONS.settings, this.translator.text("island.action.settings"), () =>
          handlers.onOpenPanel("settings")),
        close,
      ]
      : [...(recording ? [this.recordingIndicator.render(recording)] : []), close];

    return ElementFactory.create("header", {
      className: "island-head",
      children: [face, leading, ElementFactory.create("span", { className: "island-head-spacer" }), ...actions],
    });
  }

  /** On the tabs: the record button, or the session's dot and timer with pause and stop. */
  private recordingControls(recording: ActiveRecording | null, handlers: HeaderHandlers): HTMLElement[] {
    if (!recording) {
      return [ElementFactory.iconButton(ICONS.record, this.translator.text("recording.start"), () => handlers.onOpenPanel("record"))];
    }
    // A video keeps its frames' own clock, so only an audio-only session pauses.
    const pauseOrResume = recording.mode !== "audio_only"
      ? []
      : [recording.paused
        ? ElementFactory.iconButton(ICONS.play, this.translator.text("recording.resume"), handlers.onResumeRecording)
        : ElementFactory.iconButton(ICONS.pause, this.translator.text("recording.pause"), handlers.onPauseRecording)];
    return [
      this.recordingIndicator.render(recording),
      ...pauseOrResume,
      ElementFactory.iconButton(ICONS.stop, this.translator.text("recording.stop"), handlers.onStopRecording),
    ];
  }

  private tabs(tabs: readonly IslandTabLabel[], activeTab: IslandTab, onSelect: (tab: IslandTab) => void): HTMLElement {
    const buttons = tabs.map(({ tab, labelKey }) => ElementFactory.textButton({
      className: tab === activeTab ? "island-tab island-tab-on" : "island-tab",
      text: this.translator.text(labelKey),
      attributes: { role: "tab", "aria-selected": String(tab === activeTab), "data-tab": tab },
    }, () => onSelect(tab)));
    return ElementFactory.create("div", { className: "island-tabs", attributes: { role: "tablist" }, children: buttons });
  }

  private backButton(panel: TitledPanel, onOpenPanel: (panel: IslandPanel) => void): HTMLElement {
    const titleKey = HeaderView.PANEL_TITLES[panel];
    return ElementFactory.create("div", {
      className: "island-head-title",
      children: [
        ElementFactory.iconButton(ICONS.back, this.translator.text("island.action.back"), () => onOpenPanel("tabs")),
        ElementFactory.create("strong", { text: this.translator.text(titleKey) }),
      ],
    });
  }
}
