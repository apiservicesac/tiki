import { afterEach, describe, expect, it, vi } from "vitest";
import { ContentTransition, type ContentSize } from "../src/island/ContentTransition";

/** Stands in for the browser's ResizeObserver: tests decide when a size changes. */
class FakeResizeObserver {
  static latest: FakeResizeObserver | null = null;
  private watched: Element | null = null;

  constructor(private readonly callback: ResizeObserverCallback) {
    FakeResizeObserver.latest = this;
  }

  observe(target: Element): void {
    this.watched = target;
  }

  disconnect(): void {
    this.watched = null;
  }

  /** Simulates the watched element changing size. */
  resize(): void {
    if (this.watched) {
      this.callback([{ target: this.watched } as ResizeObserverEntry], this as unknown as ResizeObserver);
    }
  }
}

describe("ContentTransition", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("tells the notch when the shown content grows by itself", () => {
    vi.stubGlobal("ResizeObserver", FakeResizeObserver);
    const container = document.createElement("div");
    document.body.replaceChildren(container);
    const sizes: ContentSize[] = [];
    const transition = new ContentTransition(container, (size) => sizes.push(size));

    const content = document.createElement("div");
    transition.swap(content, false);
    Object.defineProperty(content, "offsetHeight", { value: 320 });
    Object.defineProperty(content, "offsetWidth", { value: 640 });
    FakeResizeObserver.latest?.resize();

    expect(sizes).toEqual([{ width: 640, height: 320 }]);
  });
});
