import type { AlertAction, TikiAlert } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

export interface AlertActionHandlers {
  /** Puts the alert away: the reminder already rang and is done. */
  onDismissAlert: () => void;
  /** Puts off the item's reminder. */
  onSnooze: (taskId: number, minutes: number) => void;
  /** Opens a meeting's video call. */
  onJoinMeeting: (url: string) => void;
}

/** Where the buttons go: Today's notice offers every snooze, the notch only the shortest. */
export interface AlertButtonsPlace {
  idPrefix: string;
  className: string;
  everySnooze: boolean;
}

/** The buttons an alert offers (done, snooze, join), the same wherever the alert shows. */
export class AlertActionButtons {
  constructor(private readonly translator: Translator) {}

  build(alert: TikiAlert, handlers: AlertActionHandlers, place: AlertButtonsPlace): HTMLElement[] {
    return alert.actions.flatMap((action) => this.buttons(action, alert, handlers, place));
  }

  private buttons(action: AlertAction, alert: TikiAlert, handlers: AlertActionHandlers, place: AlertButtonsPlace): HTMLElement[] {
    const { taskId, link } = alert;
    switch (action) {
      case "open":
        return [];
      case "done":
        return [this.button("alert.done", `${place.idPrefix}-done`, place, handlers.onDismissAlert)];
      case "snooze": {
        if (taskId === null) {
          return [];
        }
        const choices = place.everySnooze ? UI_CONFIG.snoozeMinutes : UI_CONFIG.snoozeMinutes.slice(0, 1);
        return choices.map(({ minutes, labelKey }) =>
          this.button(labelKey, `${place.idPrefix}-snooze-${minutes}`, place, () => handlers.onSnooze(taskId, minutes)));
      }
      case "join":
        return link === null ? [] : [this.button("meeting.join", `${place.idPrefix}-join`, place, () => handlers.onJoinMeeting(link))];
    }
  }

  private button(labelKey: string, id: string, place: AlertButtonsPlace, onPress: () => void): HTMLElement {
    return ElementFactory.textButton({ className: place.className, text: this.translator.text(labelKey), attributes: { id } }, onPress);
  }
}
