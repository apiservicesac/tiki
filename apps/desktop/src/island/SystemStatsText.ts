import type { SystemStats } from "../backend/contracts";
import { UI_CONFIG } from "../config/uiConfig";
import type { Translator } from "../i18n/Translator";

/** One figure of the computer's state as the notch shows it, and whether it runs high. */
export interface SystemFigure {
  kind: "cpu" | "memory" | "temperature" | "download";
  text: string;
  high: boolean;
}

/** Turns a measure of the computer into short figures: "CPU 23%", "RAM 61%", "54 °C", "↓ 1.2 MB/s". */
export class SystemStatsText {
  constructor(private readonly translator: Translator) {}

  figures(stats: SystemStats): SystemFigure[] {
    const limits = UI_CONFIG.systemStats;
    const cpu = Math.round(stats.cpuPercent);
    const temperature = stats.temperatureCelsius === null ? null : Math.round(stats.temperatureCelsius);
    return [
      { kind: "cpu", text: this.translator.format("system.cpu", { value: cpu }), high: cpu >= limits.highCpuPercent },
      {
        kind: "memory",
        text: this.translator.format("system.memory", { value: stats.memoryPercent }),
        high: stats.memoryPercent >= limits.highMemoryPercent,
      },
      ...(temperature === null
        ? []
        : [{
          kind: "temperature" as const,
          text: this.translator.format("system.temperature", { value: temperature }),
          high: temperature >= limits.highTemperatureCelsius,
        }]),
      { kind: "download", text: this.translator.format("system.download", { value: this.speed(stats.downloadBytesPerSecond) }), high: false },
    ];
  }

  /** "820 kB/s" or "1.2 MB/s", in the language's number format. */
  private speed(bytesPerSecond: number): string {
    const { bytesPerMegabyte, bytesPerKilobyte } = UI_CONFIG.systemStats;
    const megabytes = bytesPerSecond >= bytesPerMegabyte;
    return new Intl.NumberFormat(this.translator.localeCode, {
      style: "unit",
      unit: megabytes ? "megabyte-per-second" : "kilobyte-per-second",
      maximumFractionDigits: megabytes ? 1 : 0,
    }).format(bytesPerSecond / (megabytes ? bytesPerMegabyte : bytesPerKilobyte));
  }
}
