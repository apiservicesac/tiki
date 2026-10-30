import { UI_CONFIG } from "../config/uiConfig";
import { TagText } from "../dom/TagText";

/** What an item belongs to, shown under or beside its title: "Acme · #billing". */
export class ItemContext {
  /** The project and the tags, or `null` when the item has neither. */
  static describe(projectName: string | null, tags: readonly string[]): string | null {
    return ItemContext.join([projectName, ...tags.map(TagText.mark)]);
  }

  /** The parts that are there, in one line: "Acme · se borra en 27 días". */
  static join(parts: ReadonlyArray<string | null>): string | null {
    const present = parts.filter((part): part is string => part !== null && part !== "");
    return present.length > 0 ? present.join(UI_CONFIG.textMarks.separator) : null;
  }
}
