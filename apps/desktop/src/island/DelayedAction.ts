/** One action that runs after a delay; scheduling it again replaces the run still waiting. */
export class DelayedAction {
  private timer: ReturnType<typeof setTimeout> | null = null;

  schedule(delayMs: number, action: () => void): void {
    this.cancel();
    this.timer = setTimeout(() => {
      this.timer = null;
      action();
    }, delayMs);
  }

  cancel(): void {
    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
