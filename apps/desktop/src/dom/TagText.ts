/** Tags as the user types them ("#billing, cierre de mes") and as Tiki stores them. */
export class TagText {
  private static readonly MARK = "#";

  /** Splits on commas and "#", trims, lowercases, joins words with dashes, drops repeats. */
  static parse(text: string): string[] {
    const tags = text
      .split(/[,#]/)
      .map((tag) => tag.trim().toLowerCase().split(/\s+/).join("-"))
      .filter((tag) => tag !== "");
    return [...new Set(tags)];
  }

  /** "#billing", as a tag is shown. */
  static mark(tag: string): string {
    return `${TagText.MARK}${tag}`;
  }

  /** "#billing #cierre-de-mes", for an input box. */
  static format(tags: readonly string[]): string {
    return tags.map(TagText.mark).join(" ");
  }
}
