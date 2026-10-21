import type { HitArea, TikiBackend } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";

/**
 * Tells the shell where the island is, so clicks outside it reach other applications.
 *
 * While the island shrinks, the area stays big until the animation ends, so the pointer
 * never falls through the island in the middle of a transition.
 */
export class HitAreaReporter {
  private current: HitArea | null = null;
  private shrinkTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(private readonly backend: TikiBackend) {}

  report(target: HitArea): void {
    this.cancelShrink();
    const covering = this.current ? HitAreaReporter.union(this.current, target) : target;
    this.send(covering);
    if (!HitAreaReporter.same(covering, target)) {
      this.shrinkTimer = setTimeout(() => this.send(target), UI_CONFIG.motion.closeMs);
    }
  }

  stop(): void {
    this.cancelShrink();
  }

  private send(area: HitArea): void {
    if (this.current && HitAreaReporter.same(area, this.current)) {
      return;
    }
    this.current = area;
    void this.backend.setHitArea(area);
  }

  private cancelShrink(): void {
    if (this.shrinkTimer) {
      clearTimeout(this.shrinkTimer);
      this.shrinkTimer = null;
    }
  }

  private static union(first: HitArea, second: HitArea): HitArea {
    const left = Math.min(first.x, second.x);
    const top = Math.min(first.y, second.y);
    const right = Math.max(first.x + first.width, second.x + second.width);
    const bottom = Math.max(first.y + first.height, second.y + second.height);
    return { x: left, y: top, width: right - left, height: bottom - top };
  }

  private static same(first: HitArea, second: HitArea): boolean {
    return first.x === second.x && first.y === second.y
      && first.width === second.width && first.height === second.height;
  }
}
