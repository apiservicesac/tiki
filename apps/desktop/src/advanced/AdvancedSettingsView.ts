import type { UserPreferences } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";
import type { MascotRenderer } from "../mascot/MascotRenderer";
import { AiSettingsView, type AiSettingsHandlers } from "../settings/AiSettingsView";
import type { SettingsState } from "../settings/SettingsStore";
import { ADVANCED_SECTIONS, PLUGIN_CATALOG, type AdvancedSection } from "./advancedCatalog";
import { CalendarAccountsView, type CalendarAccountsHandlers } from "./CalendarAccountsView";
import { CatalogSectionView } from "./CatalogSectionView";
import { GeneralSectionView, type GeneralSectionHandlers } from "./GeneralSectionView";
import { ShortcutsSectionView } from "./ShortcutsSectionView";
import type { WindowControls } from "./WindowControls";

export interface AdvancedSettingsHandlers extends AiSettingsHandlers, GeneralSectionHandlers, CalendarAccountsHandlers {
  onSelectSection: (section: AdvancedSection) => void;
}

/**
 * The advanced settings window, drawn like the island: its own title bar, a side menu and
 * one section at a time. The frame is built once; only the menu state and content change.
 */
export class AdvancedSettingsView {
  private static readonly SECTION_ICONS: Readonly<Record<AdvancedSection, string>> = Object.freeze({
    general: ICONS.settings,
    ai: ICONS.sparkle,
    plugins: ICONS.plug,
    accounts: ICONS.mail,
    shortcuts: ICONS.keyboard,
  });

  private readonly general: GeneralSectionView;
  private readonly ai: AiSettingsView;
  private readonly catalog: CatalogSectionView;
  private readonly calendars: CalendarAccountsView;
  private readonly shortcuts: ShortcutsSectionView;
  private readonly titleText: HTMLElement;
  private readonly nav: HTMLElement;
  private readonly heading: HTMLElement;
  private readonly subtitle: HTMLElement;
  private readonly content: HTMLElement;

  constructor(
    private readonly root: HTMLElement,
    private readonly translator: Translator,
    mascot: MascotRenderer,
    private readonly windowControls: WindowControls,
  ) {
    this.general = new GeneralSectionView(translator);
    this.ai = new AiSettingsView(translator);
    this.catalog = new CatalogSectionView(translator);
    this.calendars = new CalendarAccountsView(translator);
    this.shortcuts = new ShortcutsSectionView(translator);
    this.titleText = ElementFactory.create("span", { className: "advanced-title" });
    this.nav = ElementFactory.create("nav", { className: "advanced-nav" });
    this.heading = ElementFactory.create("h1");
    this.subtitle = ElementFactory.create("p", { className: "advanced-subtitle" });
    this.content = ElementFactory.create("div", { className: "advanced-sections" });
    root.replaceChildren(
      this.titleBar(mascot),
      ElementFactory.create("div", {
        className: "advanced-body",
        children: [
          this.nav,
          ElementFactory.create("main", {
            className: "advanced-content",
            children: [this.heading, this.subtitle, this.content],
          }),
        ],
      }),
    );
  }

  render(
    section: AdvancedSection,
    preferences: UserPreferences,
    state: SettingsState,
    handlers: AdvancedSettingsHandlers,
  ): void {
    this.root.ownerDocument.documentElement.lang = this.translator.localeCode;
    this.titleText.textContent = this.translator.text("advanced.window_title");
    this.nav.replaceChildren(...ADVANCED_SECTIONS.map((item) => this.navButton(item, section, handlers)));
    this.heading.textContent = this.translator.text(`advanced.section.${section}`);
    this.subtitle.textContent = this.translator.text(`advanced.section.${section}.subtitle`);
    this.content.dataset.section = section;
    this.content.replaceChildren(...this.sectionContent(section, preferences, state, handlers));
  }

  private sectionContent(
    section: AdvancedSection,
    preferences: UserPreferences,
    state: SettingsState,
    handlers: AdvancedSettingsHandlers,
  ): HTMLElement[] {
    switch (section) {
      case "general":
        return this.general.render(state.autostart, state.updates, handlers);
      case "ai":
        return [this.ai.render(preferences, state.aiSettings, handlers)];
      case "plugins":
        return this.catalog.render("advanced.plugins.intro", PLUGIN_CATALOG, "advanced.plugin");
      case "accounts":
        return this.calendars.render(state.calendars, handlers);
      case "shortcuts":
        return this.shortcuts.render();
    }
  }

  private navButton(item: AdvancedSection, active: AdvancedSection, handlers: AdvancedSettingsHandlers): HTMLElement {
    const button = ElementFactory.create("button", {
      className: item === active ? "advanced-nav-item advanced-nav-item-on" : "advanced-nav-item",
      attributes: { type: "button", "data-section": item, "aria-current": String(item === active) },
      children: [
        ElementFactory.fromTrustedMarkup("advanced-nav-icon", AdvancedSettingsView.SECTION_ICONS[item]),
        ElementFactory.create("span", { text: this.translator.text(`advanced.section.${item}`) }),
      ],
    });
    button.addEventListener("click", () => handlers.onSelectSection(item));
    return button;
  }

  private titleBar(mascot: MascotRenderer): HTMLElement {
    const controls = [
      ElementFactory.iconButton(ICONS.minimize, this.translator.text("advanced.window.minimize"), () =>
        void this.windowControls.minimize()),
      ElementFactory.iconButton(ICONS.maximize, this.translator.text("advanced.window.maximize"), () =>
        void this.windowControls.toggleMaximize()),
      ElementFactory.iconButton(ICONS.close, this.translator.text("island.action.close"), () =>
        void this.windowControls.close()),
    ];
    controls[2]?.classList.add("advanced-close");
    return ElementFactory.create("header", {
      className: "advanced-titlebar",
      attributes: { "data-tauri-drag-region": "" },
      children: [
        ElementFactory.fromTrustedMarkup("advanced-mascot", mascot.render("calm", "", UI_CONFIG.faceWidth.settingsTitle)),
        this.titleText,
        ElementFactory.create("span", { className: "advanced-titlebar-spacer", attributes: { "data-tauri-drag-region": "" } }),
        ElementFactory.create("div", { className: "advanced-window-buttons", children: controls }),
      ],
    });
  }
}
