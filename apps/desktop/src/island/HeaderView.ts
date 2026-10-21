import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import { ISLAND_TABS, type IslandPanel, type IslandTab } from "./islandState";

export interface HeaderHandlers {
  onSelectTab: (tab: IslandTab) => void;
  onOpenPanel: (panel: IslandPanel) => void;
  onClose: () => void;
}

/** Top row of the expanded island: tabs on the left, actions on the right. */
export class HeaderView {
  private static readonly PANEL_TITLES: Readonly<Record<IslandPanel, string>> = Object.freeze({
    tasks: "island.tab.today",
    capture: "island.action.capture",
    settings: "settings.title",
    edit: "edit.title",
    ask: "ask.title",
  });

  constructor(private readonly translator: Translator) {}

  render(panel: IslandPanel, activeTab: IslandTab, handlers: HeaderHandlers): HTMLElement {
    const leading = panel === "tasks"
      ? this.tabs(activeTab, handlers.onSelectTab)
      : this.backButton(panel, handlers.onOpenPanel);

    return ElementFactory.create("header", {
      className: "island-head",
      children: [
        leading,
        ElementFactory.create("span", { className: "island-head-spacer" }),
        ElementFactory.iconButton(ICONS.sparkle, this.translator.text("ask.title"), () =>
          handlers.onOpenPanel("ask")),
        ElementFactory.iconButton(ICONS.plus, this.translator.text("island.action.capture"), () =>
          handlers.onOpenPanel("capture")),
        ElementFactory.iconButton(ICONS.settings, this.translator.text("island.action.settings"), () =>
          handlers.onOpenPanel("settings")),
        ElementFactory.iconButton(ICONS.close, this.translator.text("island.action.close"), handlers.onClose),
      ],
    });
  }

  private tabs(activeTab: IslandTab, onSelect: (tab: IslandTab) => void): HTMLElement {
    const buttons = ISLAND_TABS.map(({ tab, labelKey }) => {
      const button = ElementFactory.create("button", {
        className: tab === activeTab ? "island-tab island-tab-on" : "island-tab",
        text: this.translator.text(labelKey),
        attributes: { type: "button", role: "tab", "aria-selected": String(tab === activeTab), "data-tab": tab },
      });
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        onSelect(tab);
      });
      return button;
    });
    return ElementFactory.create("div", { className: "island-tabs", attributes: { role: "tablist" }, children: buttons });
  }

  private backButton(panel: IslandPanel, onOpenPanel: (panel: IslandPanel) => void): HTMLElement {
    const titleKey = HeaderView.PANEL_TITLES[panel];
    return ElementFactory.create("div", {
      className: "island-head-title",
      children: [
        ElementFactory.iconButton(ICONS.back, this.translator.text("island.action.back"), () => onOpenPanel("tasks")),
        ElementFactory.create("strong", { text: this.translator.text(titleKey) }),
      ],
    });
  }
}
