import { UI_CONFIG } from "../config/uiConfig";

/** Sizes and shares in the user's language: "120 MB", "35 %". */
export class ByteSizeText {
  constructor(private readonly localeCode: string) {}

  megabytes(bytes: number): string {
    return new Intl.NumberFormat(this.localeCode, { style: "unit", unit: "megabyte", maximumFractionDigits: 0 })
      .format(bytes / UI_CONFIG.bytesPerMegabyte);
  }

  share(part: number, whole: number): string {
    return new Intl.NumberFormat(this.localeCode, { style: "percent" }).format(whole > 0 ? part / whole : 0);
  }
}
