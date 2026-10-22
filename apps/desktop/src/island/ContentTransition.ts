import { UI_CONFIG } from "../config/uiConfig";

export interface ContentSize {
  width: number;
  height: number;
}

/**
 * Swaps what the notch shows. When the view changes, the old content fades out on its
 * own layer while the new one fades in; when only the data changes, the swap is instant
 * so refreshes never flicker.
 *
 * It also watches the shown content: when it grows or shrinks by itself (the AI fills the
 * fields of quick capture, a form switches to a reminder), `onResize` gets the new size so
 * the notch follows instead of cutting it.
 */
export class ContentTransition {
  private leavingTimer: ReturnType<typeof setTimeout> | null = null;
  private readonly watcher: ResizeObserver | null;

  constructor(
    private readonly container: HTMLElement,
    onResize: (size: ContentSize) => void,
  ) {
    this.watcher = typeof ResizeObserver === "undefined"
      ? null
      : new ResizeObserver((entries) => {
        const shown = entries.at(-1)?.target;
        if (shown instanceof HTMLElement && shown.isConnected && !shown.classList.contains("island-content-leaving")) {
          onResize(ContentTransition.sizeOf(shown));
        }
      });
  }

  /** Shows `next` and returns its natural size, which the notch animates to. */
  swap(next: HTMLElement, animate: boolean): ContentSize {
    next.classList.add("island-content");
    this.watcher?.disconnect();
    this.watcher?.observe(next);
    if (!animate) {
      this.clearLeaving();
      this.container.replaceChildren(next);
      return ContentTransition.sizeOf(next);
    }

    this.clearLeaving();
    for (const previous of Array.from(this.container.children)) {
      previous.classList.add("island-content-leaving");
    }
    next.classList.add("island-content-entering");
    this.container.append(next);
    this.leavingTimer = setTimeout(() => this.clearLeaving(), UI_CONFIG.motion.contentOutMs);
    return ContentTransition.sizeOf(next);
  }

  stop(): void {
    this.clearLeaving();
    this.watcher?.disconnect();
  }

  private clearLeaving(): void {
    if (this.leavingTimer) {
      clearTimeout(this.leavingTimer);
      this.leavingTimer = null;
    }
    this.container.querySelectorAll(".island-content-leaving").forEach((element) => element.remove());
  }

  private static sizeOf(element: HTMLElement): ContentSize {
    return { width: element.offsetWidth, height: element.offsetHeight };
  }
}
