import type { ActiveRecording, IslandSnapshot, MoodName, SystemStats, TikiAlert } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import { AccountBadge } from "../dom/AccountBadge";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { MascotRenderer } from "../mascot/MascotRenderer";
import { AlertActionButtons, type AlertActionHandlers, type AlertButtonsPlace } from "./AlertActionButtons";
import type { RecordingIndicator } from "./RecordingIndicator";
import { SystemStatsText } from "./SystemStatsText";

/**
 * Content of the folded island. Resting, it shows Tiki's face with the day's summary when
 * something needs attention, or the face alone when the day is clear. Compact adds the mood
 * word. The mood color (eyes, glow, chip) comes from the stylesheet through `data-mood`.
 *
 * The summary is written as text for the top and bottom edges and as count bubbles for the
 * left and right edges; the stylesheet shows the one that fits the orientation. While a
 * session records, the resting and compact notch keep its red dot and timer in sight.
 */
export class NotchView {
  /** The alert's buttons on the right of the notch, with only the shortest snooze. */
  private static readonly NOTCH_BUTTONS: AlertButtonsPlace = Object.freeze({ idPrefix: "notch-alert", className: "notch-alert-button", everySnooze: false });
  private readonly alertButtons: AlertActionButtons;
  private readonly statsText: SystemStatsText;

  constructor(
    private readonly mascot: MascotRenderer,
    private readonly translator: Translator,
    private readonly recordingIndicator: RecordingIndicator,
  ) {
    this.alertButtons = new AlertActionButtons(translator);
    this.statsText = new SystemStatsText(translator);
  }

  /** The resting notch while a session records: Tiki, the red dot and the timer. */
  recording(snapshot: IslandSnapshot, recording: ActiveRecording, mood: MoodName, unread: boolean): HTMLElement {
    return ElementFactory.create("div", {
      className: "notch-hidden notch-hidden-busy notch-recording",
      children: [
        this.mascotOf(snapshot, mood, UI_CONFIG.faceWidth.hidden),
        this.recordingIndicator.render(recording),
        ...this.unreadDot(unread),
      ],
    });
  }


  /**
   * The resting notch. With nothing pending it shows the computer's figures beside the face
   * (when the user wants them), so the notch is never just an empty face.
   */
  hidden(snapshot: IslandSnapshot, mood: MoodName, unread: boolean, systemStats: SystemStats | null): HTMLElement {
    const mascot = this.mascotOf(snapshot, mood, UI_CONFIG.faceWidth.hidden);
    const unreadDot = this.unreadDot(unread);
    // Hovering the resting notch during a call says why Tiki is quiet.
    const callNote: Record<string, string> = snapshot.inCall ? { title: this.translator.text("island.in_call") } : {};
    if (snapshot.attentionCount === 0) {
      return systemStats
        ? ElementFactory.create("div", {
          className: "notch-hidden notch-hidden-busy",
          attributes: callNote,
          children: [mascot, this.systemFigures(systemStats), ...unreadDot],
        })
        : ElementFactory.create("div", { className: "notch-hidden", attributes: callNote, children: [mascot, ...unreadDot] });
    }
    return ElementFactory.create("div", {
      className: "notch-hidden notch-hidden-busy",
      attributes: callNote,
      children: [
        mascot,
        this.summaryText(snapshot),
        this.countBubbles(snapshot),
        ...unreadDot,
      ],
    });
  }

  /**
   * The notch grown to announce something: Tiki's face with ripples, the kind of alert (with
   * a meeting's account), its title and one preview line of its summary and tips (cut with
   * "…"), its buttons on the right (done, the shortest snooze, join), and a thin countdown
   * until the notch folds back.
   * The aura around the notch comes from the stylesheet (`data-mode="alert"`).
   */
  alert(snapshot: IslandSnapshot, alert: TikiAlert, mood: MoodName, handlers: AlertActionHandlers): HTMLElement {
    const buttons = this.alertButtons.build(alert, handlers, NotchView.NOTCH_BUTTONS);
    const preview = [alert.summary, ...alert.tips].join(UI_CONFIG.textMarks.separator);
    const kindLabel = this.translator.text(UI_CONFIG.alertStyles[alert.kind].labelKey);
    const face = this.mascotOf(snapshot, mood, UI_CONFIG.faceWidth.alert);
    face.classList.add("notch-alert-face");
    return ElementFactory.create("div", {
      className: "notch-alert",
      attributes: { role: "status", "aria-live": "polite", "data-alert-kind": alert.kind },
      children: [
        face,
        ElementFactory.create("span", {
          className: "notch-alert-text",
          children: [
            ElementFactory.create("span", { className: "notch-alert-kind", text: kindLabel, children: this.origin(alert) }),
            ElementFactory.create("strong", { text: alert.title }),
            ElementFactory.create("span", { className: "notch-alert-preview", text: preview }),
          ],
        }),
        ...(buttons.length > 0 ? [ElementFactory.create("span", { className: "notch-alert-actions", children: buttons })] : []),
        ElementFactory.create("span", { className: "notch-alert-countdown", attributes: { "aria-hidden": "true" } }),
      ],
    });
  }

  /** A meeting's account badge, with its platform's icon, and the account's email. */
  private origin(alert: TikiAlert): HTMLElement[] {
    if (alert.origin === null) {
      return [];
    }
    const { account, platform, label } = alert.origin;
    return [ElementFactory.create("span", {
      className: "notch-alert-origin",
      attributes: { title: label },
      children: [
        AccountBadge.render(account, platform ?? account.provider),
        ElementFactory.create("span", { className: "notch-alert-email", text: account.email }),
      ],
    })];
  }

  compact(snapshot: IslandSnapshot, mood: MoodName): HTMLElement {
    return ElementFactory.create("div", {
      className: "notch-compact",
      children: [
        this.mascotOf(snapshot, mood, UI_CONFIG.faceWidth.compact),
        this.summaryText(snapshot),
        this.countBubbles(snapshot),
        ...(snapshot.recording ? [this.recordingIndicator.render(snapshot.recording)] : []),
        ElementFactory.create("span", {
          className: `notch-chip notch-chip-${mood}`,
          text: snapshot.moodLabel,
          attributes: { title: snapshot.moodLabel },
        }),
      ],
    });
  }

  private unreadDot(unread: boolean): HTMLElement[] {
    return unread ? [ElementFactory.create("span", { className: "notch-unread", attributes: { "aria-hidden": "true" } })] : [];
  }

  /** "CPU 23%  RAM 61%  54 °C  ↓ 1.2 MB/s", each figure warning when it runs high. */
  private systemFigures(stats: SystemStats): HTMLElement {
    return ElementFactory.create("span", {
      className: "notch-system",
      children: this.statsText.figures(stats).map((figure) => ElementFactory.create("span", {
        className: figure.high ? "notch-figure notch-figure-high" : "notch-figure",
        text: figure.text,
        attributes: { "data-figure": figure.kind },
      })),
    });
  }

  private mascotOf(snapshot: IslandSnapshot, mood: MoodName, size: number): HTMLElement {
    return ElementFactory.fromTrustedMarkup("notch-mascot", this.mascot.render(mood, snapshot.moodLabel, size));
  }

  private summaryText(snapshot: IslandSnapshot): HTMLElement {
    return ElementFactory.create("span", { className: "notch-summary", text: snapshot.closedSummary });
  }

  /** "3" for what is due and "2" for what is waiting; a zero count is left out. */
  private countBubbles(snapshot: IslandSnapshot): HTMLElement {
    const bubbles = [
      { count: snapshot.dueCount, kind: "due" },
      { count: snapshot.waitingCount, kind: "waiting" },
    ]
      .filter(({ count }) => count > 0)
      .map(({ count, kind }) =>
        ElementFactory.create("span", { className: `notch-count notch-count-${kind}`, text: String(count) }));

    return ElementFactory.create("span", {
      className: "notch-counts",
      attributes: { role: "img", "aria-label": snapshot.closedSummary, title: snapshot.closedSummary },
      children: bubbles,
    });
  }
}
