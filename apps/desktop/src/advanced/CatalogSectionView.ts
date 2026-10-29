import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { CatalogEntry } from "./advancedCatalog";

/**
 * A list of plugins, each with what it brings and when it arrives. The ones coming in the
 * next version get a card; the ones further away are named in one line, so the page shows
 * what is close instead of a wall of "coming soon".
 */
export class CatalogSectionView {
  constructor(private readonly translator: Translator) {}

  render(introKey: string, entries: readonly CatalogEntry[], keyPrefix: string): HTMLElement[] {
    const later = entries.filter((entry) => entry.availability === "later");
    return [
      ElementFactory.create("p", { className: "advanced-intro", text: this.translator.text(introKey) }),
      ElementFactory.create("div", {
        className: "catalog",
        children: entries.filter((entry) => entry.availability !== "later").map((entry) => this.card(entry, keyPrefix)),
      }),
      ...(later.length > 0 ? [this.laterLine(later)] : []),
    ];
  }

  /** "Later: Slack, Jira, GitHub and PostgreSQL.", joined the way the language joins lists. */
  private laterLine(entries: readonly CatalogEntry[]): HTMLElement {
    const names = new Intl.ListFormat(this.translator.localeCode, { type: "conjunction" })
      .format(entries.map((entry) => entry.name));
    return ElementFactory.create("p", { className: "catalog-later", text: this.translator.format("advanced.catalog.later", { names }) });
  }

  private card(entry: CatalogEntry, keyPrefix: string): HTMLElement {
    return ElementFactory.create("article", {
      className: "catalog-card",
      attributes: { "data-entry": entry.id },
      children: [
        ElementFactory.create("span", { className: "catalog-initial", text: entry.name.charAt(0) }),
        ElementFactory.create("div", {
          className: "catalog-text",
          children: [
            ElementFactory.create("div", {
              className: "catalog-title",
              children: [
                ElementFactory.create("strong", { text: entry.name }),
                ElementFactory.create("span", {
                  className: `catalog-badge catalog-badge-${entry.availability}`,
                  text: this.translator.text(`advanced.availability.${entry.availability}`),
                }),
              ],
            }),
            ElementFactory.create("p", { text: this.translator.text(`${keyPrefix}.${entry.id}`) }),
            ElementFactory.create("div", {
              className: "catalog-kinds",
              children: entry.kinds.map((kind) =>
                ElementFactory.create("span", { className: `catalog-kind catalog-kind-${kind}`, text: this.translator.text(`advanced.kind.${kind}`) })),
            }),
          ],
        }),
      ],
    });
  }
}
