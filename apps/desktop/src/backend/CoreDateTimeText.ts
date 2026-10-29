/**
 * Dates and times as the core writes them, in local time: "2026-10-29T09:30:00" for a
 * moment and "08:30:00" for a time of day. The pickers show minutes; the core keeps seconds.
 */
export class CoreDateTimeText {
  private static readonly DAY_LENGTH = "2026-10-29".length;
  private static readonly TIME_START = "2026-10-29T".length;
  private static readonly MINUTES_LENGTH = "09:30".length;
  private static readonly PICKER_LENGTH = "2026-10-29T09:30".length;
  private static readonly WHOLE_MINUTE = ":00";
  private static readonly MILLISECONDS_PER_DAY = 86_400_000;

  /** "2026-10-29" from a moment. */
  static day(moment: string): string {
    return moment.slice(0, CoreDateTimeText.DAY_LENGTH);
  }

  /** "09:30" from a moment. */
  static time(moment: string): string {
    return CoreDateTimeText.clock(moment.slice(CoreDateTimeText.TIME_START));
  }

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

  /** The local midnight that starts a day ("2026-10-29"). */
  static startOf(day: string): Date {
    const [year, month, date] = day.split("-").map(Number) as [number, number, number];
    return new Date(year, month - 1, date);
  }

  /** Whole days from the start of `today` to a day: 0 for today, 1 for tomorrow. */
  static daysFrom(today: Date, day: string): number {
    const startOfToday = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    return Math.round((CoreDateTimeText.startOf(day).getTime() - startOfToday.getTime()) / CoreDateTimeText.MILLISECONDS_PER_DAY);
  }
}
