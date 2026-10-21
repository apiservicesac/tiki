/** Loose text search for lists: ignores case and accents, so "almacen" finds "Almacén". */
export class TextMatcher {
  static normalize(text: string): string {
    return text.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().trim();
  }

  /** Whether `text` contains `query`; an empty query matches everything. */
  static matches(text: string, query: string): boolean {
    const normalizedQuery = TextMatcher.normalize(query);
    return normalizedQuery === "" || TextMatcher.normalize(text).includes(normalizedQuery);
  }
}
