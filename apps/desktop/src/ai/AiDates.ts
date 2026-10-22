/** Dates as the models read and write them: ISO, in the user's local time. */
export class AiDates {
  private static readonly DATE = /^\d{4}-\d{2}-\d{2}$/;
  private static readonly DATE_TIME = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}$/;

  /** "Wednesday, 2026-10-21 17:30". */
  static describeNow(now: Date): string {
    const weekday = now.toLocaleDateString("en-US", { weekday: "long" });
    return `${weekday}, ${AiDates.isoDate(now)} ${AiDates.pad(now.getHours())}:${AiDates.pad(now.getMinutes())}`;
  }

  static isoDate(day: Date): string {
    return [day.getFullYear(), AiDates.pad(day.getMonth() + 1), AiDates.pad(day.getDate())].join("-");
  }

  /** The date if the model wrote YYYY-MM-DD, otherwise `null`. */
  static date(text: string | null): string | null {
    return text && AiDates.DATE.test(text) ? text : null;
  }

  /** `YYYY-MM-DDTHH:MM:00` if the model wrote YYYY-MM-DDTHH:MM, otherwise `null`. */
  static dateTime(text: string | null): string | null {
    return text && AiDates.DATE_TIME.test(text) ? `${text}:00` : null;
  }

  private static pad(value: number): string {
    return String(value).padStart(2, "0");
  }
}
