import type { Translator } from "../i18n/Translator";
import { SettingRows } from "../settings/SettingRows";
import { UpdateSettingsView, type UpdateSettingsHandlers, type UpdateState } from "../settings/UpdateSettingsView";

export interface GeneralSectionHandlers extends UpdateSettingsHandlers {
  onSetAutostart: (enabled: boolean) => void;
}

/** Start with Windows, the installed version and updates. */
export class GeneralSectionView {
  private readonly updates: UpdateSettingsView;
  private readonly rows: SettingRows;

  constructor(translator: Translator) {
    this.updates = new UpdateSettingsView(translator);
    this.rows = new SettingRows(translator);
  }

  render(autostart: boolean, updates: UpdateState, handlers: GeneralSectionHandlers): HTMLElement[] {
    return [
      this.rows.section("advanced.general.startup", [
        this.rows.switchRow({ labelKey: "settings.autostart", hintKey: "advanced.general.startup_hint" },
          autostart, (enabled) => handlers.onSetAutostart(enabled)),
      ]),
      this.updates.render(updates, handlers),
    ];
  }

}
