import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import type { IslandPanel, IslandTab } from "./islandState";

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
}

/**
 * Top row of the expanded island: Tiki's face, then the tabs (or a back button and the
 * panel's title), and on the right the actions that make sense there. On the tabs: new and
 * settings; inside a panel only close, since the back button already leads out.
 */
export class HeaderView {
  private static readonly PANEL_TITLES: Readonly<Record<TitledPanel, string>> = Object.freeze({
    capture: "island.action.new",
    settings: "settings.title",
    edit: "edit.title",
    projects: "projects.title",
    brief: "receipt.title",
  });

  constructor(private readonly translator: Translator) {}

  render(
    face: HTMLElement,
    panel: IslandPanel,
    tabs: readonly IslandTabLabel[],
    activeTab: IslandTab,
    handlers: HeaderHandlers,
  ): HTMLElement {
    const leading = panel === "tabs"
      ? this.tabs(tabs, activeTab, handlers.onSelectTab)
      : this.backButton(panel, handlers.onOpenPanel);
    const close = ElementFactory.iconButton(ICONS.close, this.translator.text("island.action.close"), handlers.onClose);
    const actions = panel === "tabs"
      ? [
        ElementFactory.iconButton(ICONS.plus, this.translator.text("island.action.new"), () =>
          handlers.onOpenPanel("capture")),
        ElementFactory.iconButton(ICONS.settings, this.translator.text("island.action.settings"), () =>
          handlers.onOpenPanel("settings")),
        close,
      ]
      : [close];

    return ElementFactory.create("header", {
      className: "island-head",
      children: [face, leading, ElementFactory.create("span", { className: "island-head-spacer" }), ...actions],
    });
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
