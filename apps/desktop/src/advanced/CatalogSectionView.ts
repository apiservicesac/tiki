import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import type { CatalogEntry } from "./advancedCatalog";

/** A list of plugins or accounts, each with what it brings and when it arrives. */
export class CatalogSectionView {
  constructor(private readonly translator: Translator) {}

  render(introKey: string, entries: readonly CatalogEntry[], keyPrefix: string): HTMLElement[] {
    return [
      ElementFactory.create("p", { className: "advanced-intro", text: this.translator.text(introKey) }),
      ElementFactory.create("div", {
        className: "catalog",
        children: entries.map((entry) => this.card(entry, keyPrefix)),
      }),
    ];
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
