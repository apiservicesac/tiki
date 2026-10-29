import { UI_CONFIG } from "../config/uiConfig";
import { TagText } from "../dom/TagText";

/** What an item belongs to, shown under or beside its title: "Altamira · #billing". */
export class ItemContext {
  /** The project and the tags, or `null` when the item has neither. */
  static describe(projectName: string | null, tags: readonly string[]): string | null {
    const parts = [...(projectName ? [projectName] : []), ...tags.map(TagText.mark)];
    return parts.length > 0 ? parts.join(UI_CONFIG.textMarks.separator) : null;
  }
}
