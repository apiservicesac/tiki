/**
 * Keeps the keyboard focus across a redraw. Views that are kept between frames (the search
 * boxes, the capture box) are moved into new containers, and moving an element drops its
 * focus; this puts the focus and the caret back where they were.
 */
export class FocusKeeper {
  private constructor(
    private readonly element: HTMLElement | null,
    private readonly selection: [number, number] | null,
  ) {}

  static remember(document: Document): FocusKeeper {
    const element = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const selection = element instanceof HTMLInputElement && FocusKeeper.hasCaret(element)
      ? [element.selectionStart ?? 0, element.selectionEnd ?? 0] as [number, number]
      : null;
    return new FocusKeeper(element, selection);
  }

  restore(): void {
    const element = this.element;
    if (!element || !element.isConnected || element.ownerDocument.activeElement === element) {
      return;
    }
    element.focus();
    if (this.selection && element instanceof HTMLInputElement) {
      element.setSelectionRange(...this.selection);
    }
  }

  /** Only text-like inputs have a caret; asking a date input for one throws. */
  private static hasCaret(input: HTMLInputElement): boolean {
    return ["text", "search", "password", "url", "tel", "email"].includes(input.type);
  }
}
