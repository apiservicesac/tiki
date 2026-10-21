/** Escapes text before it goes inside generated SVG or HTML markup. */
export class MarkupEscaper {
  private static readonly replacements: Readonly<Record<string, string>> = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  };

  static escape(text: string): string {
    return text.replace(/[&<>"']/g, (character) => MarkupEscaper.replacements[character] ?? character);
  }
}
