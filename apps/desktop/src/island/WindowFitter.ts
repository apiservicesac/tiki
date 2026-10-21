import type { TikiBackend } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";

export interface IslandSize {
  width: number;
  height: number;
}

/**
 * Keeps the desktop window as big as the island, so the transparent space around it
 * never blocks clicks on other applications.
 *
 * The window grows before the island opens and shrinks only after the island has
 * finished folding, so the animation is never cut by the window edge.
 */
export class WindowFitter {
  private current: IslandSize = { width: 0, height: 0 };
  private shrinkTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(private readonly backend: TikiBackend) {}

  request(target: IslandSize): void {
    if (target.width === 0 || target.height === 0) {
      return;
    }
    this.cancelShrink();
    const roomForAnimation = {
      width: Math.max(target.width, this.current.width),
      height: Math.max(target.height, this.current.height),
    };
    this.apply(roomForAnimation);
    if (!WindowFitter.same(roomForAnimation, target)) {
      this.shrinkTimer = setTimeout(() => this.apply(target), UI_CONFIG.islandTransitionMs);
    }
  }

  stop(): void {
    this.cancelShrink();
  }

  private apply(size: IslandSize): void {
    if (WindowFitter.same(size, this.current)) {
      return;
    }
    this.current = size;
    void this.backend.fitWindow(size.width, size.height);
  }

  private cancelShrink(): void {
    if (this.shrinkTimer) {
      clearTimeout(this.shrinkTimer);
      this.shrinkTimer = null;
    }
  }

  private static same(first: IslandSize, second: IslandSize): boolean {
    return first.width === second.width && first.height === second.height;
  }
}
