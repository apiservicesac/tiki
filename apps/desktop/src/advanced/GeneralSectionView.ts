import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { UpdateSettingsView, type UpdateSettingsHandlers, type UpdateState } from "../settings/UpdateSettingsView";

export interface GeneralSectionHandlers extends UpdateSettingsHandlers {
  onSetAutostart: (enabled: boolean) => void;
}

/** Start with Windows, the installed version and updates. */
export class GeneralSectionView {
  private readonly updates: UpdateSettingsView;

  constructor(private readonly translator: Translator) {
    this.updates = new UpdateSettingsView(translator);
  }

  render(autostart: boolean, updates: UpdateState, handlers: GeneralSectionHandlers): HTMLElement[] {
    const toggle = ElementFactory.create("button", {
      className: "settings-option",
      text: this.translator.text("settings.autostart"),
      attributes: { type: "button", role: "switch", "aria-checked": String(autostart), "data-setting": "settings.autostart" },
    });
    toggle.addEventListener("click", () => handlers.onSetAutostart(!autostart));
    return [
      ElementFactory.create("section", {
        className: "settings-group",
        children: [
          ElementFactory.create("h3", { text: this.translator.text("advanced.general.startup") }),
          ElementFactory.create("div", { className: "settings-options", children: [toggle] }),
          ElementFactory.create("p", { className: "settings-note", text: this.translator.text("advanced.general.startup_hint") }),
        ],
      }),
      this.updates.render(updates, handlers),
    ];
  }
}
