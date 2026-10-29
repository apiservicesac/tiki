import type { AssistantActions } from "../assistant/AssistantActions";
import type { TikiAlert } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { AlertSound } from "./AlertSound";
import type { IslandModeMachine } from "./IslandModeMachine";
import type { IslandStore } from "./IslandStore";
import type { ItemActions } from "./ItemActions";

/** Alerts from the core: announcing them, opening what they are about, putting them away. */
export class AlertFlow {
  private readonly sound = new AlertSound();

  constructor(
    private readonly store: IslandStore,
    private readonly modes: IslandModeMachine,
    private readonly items: ItemActions,
    private readonly assistantActions: AssistantActions,
  ) {}

  /**
   * The notch grows with Tiki, the title and a preview, and chimes, then folds leaving a dot
   * until the island is opened. If the island is already open, the alert just appears above
   * the Today list.
   */
  async announce(alert: TikiAlert): Promise<void> {
    await this.store.refresh();
    void this.assistantActions.prepareDayBrief();
    const alreadyOpen = this.store.state().mode === "expanded";
    this.store.update({ alert, unreadAlert: !alreadyOpen });
    if (this.store.preferences().alertSound) {
      this.sound.play();
    }
    if (alreadyOpen) {
      this.store.react(UI_CONFIG.alertStyles[alert.kind].mood);
    } else {
      this.modes.send("alertArrived");
    }
  }

  /** Opens what the alert is about: its task, the Agenda for a meeting, or the brief as a receipt. */
  async open(): Promise<void> {
    const alert = this.store.state().alert;
    this.modes.cancel();
    if (alert && alert.taskId !== null) {
      await this.items.open(alert.taskId);
    } else if (alert?.kind === "meeting") {
      this.store.update({ mode: "expanded", panel: "tabs", tab: "agenda" });
    } else {
      this.store.update({ mode: "expanded", panel: alert ? "brief" : "tabs" });
    }
  }

  dismiss(): void {
    this.store.update({ alert: null, unreadAlert: false });
  }
}
