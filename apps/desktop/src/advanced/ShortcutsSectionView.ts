import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";
import { SHORTCUTS } from "./advancedCatalog";

/** The global shortcuts Tiki listens to. */
export class ShortcutsSectionView {
  constructor(private readonly translator: Translator) {}

  render(): HTMLElement[] {
    return [
      ElementFactory.create("div", {
        className: "shortcuts",
        children: SHORTCUTS.map((shortcut) =>
          ElementFactory.create("div", {
            className: "shortcut-row",
            children: [
              ElementFactory.create("span", { text: this.translator.text(shortcut.labelKey) }),
              ElementFactory.create("span", {
                className: "shortcut-keys",
                children: shortcut.keys.map((key) =>
                  ElementFactory.create("kbd", { text: this.translator.text(`advanced.key.${key.toLowerCase()}`) })),
              }),
            ],
          })),
      }),
    ];
  }
}
