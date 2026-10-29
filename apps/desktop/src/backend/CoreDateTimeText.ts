/**
 * Dates and times as the core writes and reads them, in local time: "2026-10-29T09:30:00"
 * for a moment and "08:30:00" for a time of day. The pickers show minutes; the core keeps
 * seconds. This is the one place that converts between the two.
 */
export class CoreDateTimeText {
  private static readonly MINUTES_LENGTH = "09:30".length;
  private static readonly PICKER_LENGTH = "2026-10-29T09:30".length;
  private static readonly WHOLE_MINUTE = ":00";

  /** "08:30" from a time of day, for a time picker. */
  static clock(timeOfDay: string): string {
    return timeOfDay.slice(0, CoreDateTimeText.MINUTES_LENGTH);
  }

  /** The time of day a time picker shows ("08:30"), as the core expects it. */
  static fromClock(value: string): string {
    return `${value}${CoreDateTimeText.WHOLE_MINUTE}`;
  }

  /** "2026-10-29T09:30" for a date-and-time picker; empty when there is no moment. */
  static toPicker(moment: string | null): string {
    return moment?.slice(0, CoreDateTimeText.PICKER_LENGTH) ?? "";
  }

  /** The moment a date-and-time picker shows, as the core expects it; `null` when empty. */
  static fromPicker(value: string): string | null {
    return value === "" ? null : `${value}${CoreDateTimeText.WHOLE_MINUTE}`;
  }
}
