/** A duration as a clock: "04:07", or "1:02:09" past an hour. */
export class DurationText {
  private static readonly MS_PER_SECOND = 1_000;
  private static readonly SECONDS_PER_MINUTE = 60;
  private static readonly SECONDS_PER_HOUR = 3_600;
  private static readonly CLOCK_DIGITS = 2;
  private static readonly CLOCK_SEPARATOR = ":";

  static clock(milliseconds: number): string {
    const totalSeconds = Math.max(0, Math.floor(milliseconds / DurationText.MS_PER_SECOND));
    const hours = Math.floor(totalSeconds / DurationText.SECONDS_PER_HOUR);
    const minutes = Math.floor((totalSeconds % DurationText.SECONDS_PER_HOUR) / DurationText.SECONDS_PER_MINUTE);
    const seconds = totalSeconds % DurationText.SECONDS_PER_MINUTE;
    const pad = (value: number) => String(value).padStart(DurationText.CLOCK_DIGITS, "0");
    const clock = [pad(minutes), pad(seconds)];
    return (hours > 0 ? [String(hours), ...clock] : clock).join(DurationText.CLOCK_SEPARATOR);
  }

  /** Seconds, as a media element counts them. */
  static seconds(milliseconds: number): number {
    return milliseconds / DurationText.MS_PER_SECOND;
  }

  static milliseconds(seconds: number): number {
    return seconds * DurationText.MS_PER_SECOND;
  }
}
