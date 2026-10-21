import english from "../../../../locales/en.json";
import spanish from "../../../../locales/es.json";
import type { LocaleCode } from "./LocaleCode";

interface LocaleCatalog {
  messages: Record<string, string>;
}

/** Turns message keys into text, using the same catalogs as the Rust core. */
export class Translator {
  private static readonly catalogs: Record<LocaleCode, LocaleCatalog> = {
    es: spanish,
    en: english,
  };

  constructor(private readonly locale: LocaleCode) {}

  get localeCode(): LocaleCode {
    return this.locale;
  }

  /** Text for a key; the key itself when missing, so gaps are visible but harmless. */
  text(key: string): string {
    return Translator.catalogs[this.locale].messages[key] ?? key;
  }

  /** Text with `{name}` placeholders replaced by the given values. */
  format(key: string, values: Record<string, string | number>): string {
    return Object.entries(values).reduce(
      (text, [name, value]) => text.replaceAll(`{${name}}`, String(value)),
      this.text(key),
    );
  }
}
