import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import type { Translator } from "../i18n/Translator";

/** What one line of a list shows, from left to right. */
export interface ItemRowContent {
  /** Classes added to `island-row`, such as `agenda-row`. */
  variant?: string;
  attributes: Record<string, string>;
  /** The project's color bar, or the time in the Agenda. */
  lead: HTMLElement;
  /** Drawn before the title, such as the priority mark. */
  mark?: HTMLElement | null;
  title: string;
  /** A smaller line under the title: the project and tags, a repeat, a place. */
  context: string | null;
  /** Opens the item; without it the title is plain text. */
  onOpen?: () => void;
  /** When it is due, or an icon of what it is. */
  badge: HTMLElement;
  /** The action at the end of the line, if any: deliver, delivered, join. */
  trailing: HTMLElement | null;
}

/**
 * One line of a list of items, the same in Today, Tasks, the delivered history and the
 * Agenda: a lead, the title with its context, a badge and an action.
 */
export class ItemRow {
  constructor(private readonly translator: Translator) {}

  render(content: ItemRowContent): HTMLElement {
    return ElementFactory.create("div", {
      className: content.variant ? `island-row ${content.variant}` : "island-row",
      attributes: content.attributes,
      children: [
        content.lead,
        this.title(content),
        content.badge,
        ...(content.trailing ? [content.trailing] : []),
      ],
    });
  }

  /** The project's color; items without a project keep the stylesheet's faint bar. */
  colorBar(color: string | null): HTMLElement {
    const bar = ElementFactory.create("span", { className: "island-row-bar" });
    if (color !== null) {
      bar.style.background = color;
    }
    return bar;
  }

  /** The time an item happens, leading an Agenda line. */
  time(text: string): HTMLElement {
    return ElementFactory.create("span", { className: "agenda-time", text });
  }

  /** "mañana", "1 día de atraso": a pill colored by `tone` (an urgency, or `done`). */
  when(tone: string, label: string): HTMLElement {
    return ElementFactory.create("span", { className: `island-when island-when-${tone}`, text: label });
  }

  /** A bell for a reminder, a calendar for a meeting. */
  kindIcon(iconMarkup: string): HTMLElement {
    return ElementFactory.fromTrustedMarkup("agenda-bell", iconMarkup);
  }

  /** The check that delivers the item: always there, faint until the row is pointed at. */
  deliverButton(onDeliver: () => void): HTMLElement {
    const button = ElementFactory.iconButton(ICONS.check, this.translator.text("island.row.done"), onDeliver);
    button.classList.add("island-row-done");
    return button;
  }

  /** "Reopen", on an item already delivered. */
  reopenButton(onReopen: () => void): HTMLElement {
    return ElementFactory.textButton({
      className: "island-row-reopen",
      text: this.translator.text("island.row.reopen"),
    }, onReopen);
  }

  private title(content: ItemRowContent): HTMLElement {
    const children = [
      ...(content.mark ? [content.mark] : []),
      document.createTextNode(content.title),
      ...(content.context ? [ElementFactory.create("small", { text: content.context })] : []),
    ];
    if (!content.onOpen) {
      return ElementFactory.create("span", { className: "island-row-text", children });
    }
    return ElementFactory.textButton({
      className: "island-row-text",
      attributes: { title: this.translator.text("island.row.edit") },
      children,
    }, content.onOpen);
  }
}
