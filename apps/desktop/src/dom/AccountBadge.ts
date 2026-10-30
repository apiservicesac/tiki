import type { AccountIdentity, CalendarProvider, MeetingPlatform } from "../backend/contracts";
import { ElementFactory } from "./ElementFactory";
import { ICONS } from "./iconLibrary";

/** What an account badge shows the icon of: a meeting's platform, or the account's provider. */
export type AccountService = MeetingPlatform | CalendarProvider;

/**
 * Whose account something comes from: the provider's picture of the account with the
 * service's icon on its corner, or the service's icon alone while there is no picture.
 * Its size comes from `--account-badge-size` where it is placed.
 */
export class AccountBadge {
  private static readonly SERVICE_ICONS: Readonly<Record<AccountService, string>> = Object.freeze({
    google_meet: ICONS.googleMeet,
    teams: ICONS.teams,
    zoom: ICONS.zoom,
    other: ICONS.video,
    google: ICONS.google,
    microsoft: ICONS.outlook,
  });

  static render(account: AccountIdentity, service: AccountService): HTMLElement {
    const icon = ElementFactory.fromTrustedMarkup("account-service", AccountBadge.SERVICE_ICONS[service]);
    icon.dataset.service = service;
    const picture = account.avatar === null
      ? []
      : [ElementFactory.create("img", { className: "account-avatar", attributes: { src: account.avatar, alt: "" } })];
    return ElementFactory.create("span", { className: "account-badge", children: [...picture, icon] });
  }
}
