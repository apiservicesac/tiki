import { CALENDAR_PROVIDERS, type CalendarAccount, type CalendarProvider, type CalendarStatus } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { AccountBadge } from "../dom/AccountBadge";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

/** What the accounts section shows besides the accounts themselves. */
export interface CalendarSettingsState {
  status: CalendarStatus;
  /** The provider whose sign-in is open in the browser, if any. */
  connecting: CalendarProvider | null;
  failed: boolean;
}

export interface CalendarAccountsHandlers {
  onConnectCalendar: (provider: CalendarProvider) => void;
  onCancelCalendarSignIn: () => void;
  onDisconnectCalendar: (accountId: number) => void;
}

/**
 * Connected calendars, each with its picture, name, email and provider, and the buttons to
 * add one. Tiki only reads them: meetings show in the Agenda and Tiki warns before each one.
 */
export class CalendarAccountsView {
  constructor(private readonly translator: Translator) {}

  render(state: CalendarSettingsState, handlers: CalendarAccountsHandlers): HTMLElement[] {
    return [
      ElementFactory.create("p", { className: "advanced-intro", text: this.translator.text("advanced.accounts.intro") }),
      ElementFactory.create("section", {
        className: "settings-group settings-calendars",
        children: [
          ElementFactory.create("h3", { text: this.translator.text("calendar.connected") }),
          ...this.accounts(state.status.accounts, handlers),
        ],
      }),
      ElementFactory.create("section", {
        className: "settings-group",
        children: [
          ElementFactory.create("h3", { text: this.translator.text("calendar.add") }),
          ElementFactory.create("div", {
            className: "settings-options",
            children: [
              ...CALENDAR_PROVIDERS.map((provider) => this.connectButton(provider, state, handlers)),
              ...this.cancelButton(state, handlers),
            ],
          }),
          ...this.notes(state),
        ],
      }),
    ];
  }

  private accounts(accounts: readonly CalendarAccount[], handlers: CalendarAccountsHandlers): HTMLElement[] {
    if (accounts.length === 0) {
      return [ElementFactory.create("p", { className: "settings-note", text: this.translator.text("calendar.none") })];
    }
    return accounts.map((account) => {
      const remove = ElementFactory.create("button", {
        className: "settings-button settings-button-quiet",
        text: this.translator.text("calendar.disconnect"),
        attributes: { type: "button", "data-account": String(account.accountId) },
      });
      remove.addEventListener("click", () => handlers.onDisconnectCalendar(account.accountId));
      const provider = this.translator.text(`calendar.provider.${account.provider}`);
      const details = account.displayName === "" ? [provider] : [account.email, provider];
      return ElementFactory.create("div", {
        className: "settings-row calendar-account",
        children: [
          AccountBadge.render(account, account.provider),
          ElementFactory.create("span", {
            className: "settings-connected",
            text: account.displayName || account.email,
            children: [ElementFactory.create("small", { text: details.join(UI_CONFIG.textMarks.separator) })],
          }),
          remove,
        ],
      });
    });
  }

  private connectButton(provider: CalendarProvider, state: CalendarSettingsState, handlers: CalendarAccountsHandlers): HTMLElement {
    const available = state.status.available.includes(provider);
    const connecting = state.connecting === provider;
    const button = ElementFactory.create("button", {
      className: "settings-option",
      text: this.translator.text(connecting ? "calendar.connecting" : `calendar.connect.${provider}`),
      attributes: { type: "button", "data-provider": provider },
    });
    button.disabled = !available || connecting;
    button.addEventListener("click", () => handlers.onConnectCalendar(provider));
    return button;
  }

  /** While the browser is open, a way out for when the user closed it. */
  private cancelButton(state: CalendarSettingsState, handlers: CalendarAccountsHandlers): HTMLElement[] {
    if (state.connecting === null) {
      return [];
    }
    const cancel = ElementFactory.create("button", {
      className: "settings-button settings-button-quiet calendar-cancel",
      text: this.translator.text("calendar.cancel"),
      attributes: { type: "button" },
    });
    cancel.addEventListener("click", () => handlers.onCancelCalendarSignIn());
    return [cancel];
  }

  private notes(state: CalendarSettingsState): HTMLElement[] {
    const keys = [
      ...(state.connecting !== null ? ["calendar.browser_hint"] : []),
      ...(state.failed ? ["calendar.failed"] : []),
      ...(state.status.available.length < CALENDAR_PROVIDERS.length ? ["calendar.unavailable"] : []),
      "calendar.privacy",
    ];
    return keys.map((key) => ElementFactory.create("p", { className: "settings-note", text: this.translator.text(key) }));
  }
}
