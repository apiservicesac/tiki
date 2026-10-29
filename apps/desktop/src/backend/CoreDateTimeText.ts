/**
 * Dates and times as the core writes them, in local time: "2026-10-29T09:30:00" for a
 * moment and "08:30:00" for a time of day. The pickers show minutes; the core keeps seconds.
 */
export class CoreDateTimeText {
  private static readonly MINUTES_LENGTH = "09:30".length;
  private static readonly PICKER_LENGTH = "2026-10-29T09:30".length;
  private static readonly WHOLE_MINUTE = ":00";

  /** "08:30" from a time of day, for a time picker. */
  static clock(timeOfDay: string): string {
    return timeOfDay.slice(0, CoreDateTimeText.MINUTES_LENGTH);
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
