import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export type UpdateStatus = "idle" | "checking" | "up_to_date" | "available" | "installing" | "failed";

export interface UpdateState {
  currentVersion: string;
  status: UpdateStatus;
  /** The newer version, when one is available. */
  availableVersion: string | null;
}

export interface UpdateSettingsHandlers {
  onCheckUpdate: () => void;
  onInstallUpdate: () => void;
}

/** The installed version, a button to look for a new one and another to install it. */
export class UpdateSettingsView {
  private static readonly STATUS_KEYS: Readonly<Partial<Record<UpdateStatus, string>>> = Object.freeze({
    checking: "settings.updates.checking",
    up_to_date: "settings.updates.up_to_date",
    installing: "settings.updates.installing",
    failed: "settings.updates.failed",
  });

  constructor(private readonly translator: Translator) {}

  render(state: UpdateState, handlers: UpdateSettingsHandlers): HTMLElement {
    const statusKey = UpdateSettingsView.STATUS_KEYS[state.status];
    return ElementFactory.create("section", {
      className: "settings-group settings-updates",
      children: [
        ElementFactory.create("h3", { text: this.translator.text("settings.updates") }),
        ElementFactory.create("div", {
          className: "settings-row",
          children: [
            ElementFactory.create("span", {
              className: "settings-row-label",
              text: this.translator.format("settings.updates.current", { version: state.currentVersion }),
            }),
            this.action(state, handlers),
          ],
        }),
        ...(statusKey
          ? [ElementFactory.create("p", { className: "settings-note", text: this.translator.text(statusKey) })]
          : []),
      ],
    });
  }

  private action(state: UpdateState, handlers: UpdateSettingsHandlers): HTMLElement {
    const canInstall = state.status === "available" && state.availableVersion !== null;
    const button = ElementFactory.create("button", {
      className: canInstall ? "settings-button" : "settings-button settings-button-quiet",
      text: canInstall
        ? this.translator.format("settings.updates.install", { version: state.availableVersion ?? "" })
        : this.translator.text("settings.updates.check"),
      attributes: { type: "button", id: "update-action" },
    });
    button.disabled = state.status === "checking" || state.status === "installing";
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      if (canInstall) {
        handlers.onInstallUpdate();
      } else {
        handlers.onCheckUpdate();
      }
    });
    return button;
  }
}
