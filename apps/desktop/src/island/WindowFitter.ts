import type { TikiBackend } from "../backend/contracts";

/**
 * Keeps the desktop window exactly as big as the island, so the transparent space
 * around it never blocks clicks on other applications.
 */
export class WindowFitter {
  private observer: ResizeObserver | null = null;
  private lastSize = "";

  constructor(private readonly backend: TikiBackend) {}

  observe(element: HTMLElement): void {
    if (typeof ResizeObserver === "undefined") {
      return;
    }
    this.observer = new ResizeObserver(() => this.fit(element));
    this.observer.observe(element);
  }

  disconnect(): void {
    this.observer?.disconnect();
    this.observer = null;
  }

  private fit(element: HTMLElement): void {
    const width = Math.ceil(element.scrollWidth);
    const height = Math.ceil(element.scrollHeight);
    const size = `${width}x${height}`;
    if (size === this.lastSize || width === 0 || height === 0) {
      return;
    }
    this.lastSize = size;
    void this.backend.fitWindow(width, height);
  }
}
