import { readdirSync, readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { describe, expect, it } from "vitest";
import english from "../../../locales/en.json";
import spanish from "../../../locales/es.json";
import { ADVANCED_SECTIONS, PLUGIN_CATALOG, SHORTCUTS } from "../src/advanced/advancedCatalog";
import {
  AI_PROVIDERS,
  CALENDAR_PROVIDERS,
  ITEM_KINDS,
  PRIORITIES,
  RECORDING_MODES,
  REPEATS,
  SESSION_STATUSES,
  SPEAKERS,
  TRANSCRIPTION_QUALITIES,
  VIDEO_QUALITIES,
  TASK_STATUSES,
  USER_ERROR_KEYS,
} from "../src/backend/contracts";
import { UI_CONFIG } from "../src/config/uiConfig";
import { TASKS_VIEWS } from "../src/island/islandState";

const CATALOGS = { es: spanish.messages, en: english.messages } as Record<string, Record<string, string>>;
const KEYS = new Set(Object.keys(english.messages));
const NAMESPACES = new Set([...KEYS].map((key) => key.split(".")[0]));
/** "edit.field.title": dotted lowercase words, like every catalog key. */
const KEY_SHAPE = /^[a-z][a-z0-9_]*(\.[a-z0-9_]+)+$/;

/** Every TypeScript file under a folder, recursively. */
function sourceFiles(folder: string): string[] {
  return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) {
      return sourceFiles(path);
    }
    return path.endsWith(".ts") ? [path] : [];
  });
}

/** String literals in the interface's code that look like a key of the catalogs. */
function literalKeys(): string[] {
  const literals = sourceFiles(resolve("src")).flatMap((file) =>
    [...readFileSync(file, "utf8").matchAll(/"([^"\n]+)"/g)].map((match) => match[1] ?? ""));
  return [...new Set(literals)].filter((text) => KEY_SHAPE.test(text) && NAMESPACES.has(text.split(".")[0] ?? ""));
}

/** A literal names a key, or the prefix of a family of keys (`advanced.plugin` → `advanced.plugin.odoo`). */
function isKnown(literal: string): boolean {
  return KEYS.has(literal) || [...KEYS].some((key) => key.startsWith(`${literal}.`));
}

/** Keys the interface builds from a list of values (`priority.${priority}`). */
function builtKeys(): string[] {
  return [
    ...PRIORITIES.map((priority) => `priority.${priority}`),
    ...REPEATS.map((repeat) => `repeat.${repeat}`),
    ...TASK_STATUSES.map((status) => `status.${status}`),
    ...ITEM_KINDS.map((kind) => `kind.${kind}`),
    ...TASKS_VIEWS.flatMap((view) => [`tasks.view.${view}`, `tasks.empty_${view}`]),
    ...AI_PROVIDERS.map((provider) => `provider.${provider}`),
    ...CALENDAR_PROVIDERS.map((provider) => `calendar.provider.${provider}`),
    ...ADVANCED_SECTIONS.flatMap((section) => [`advanced.section.${section}`, `advanced.section.${section}.subtitle`]),
    ...PLUGIN_CATALOG.flatMap((entry) => [
      `advanced.plugin.${entry.id}`,
      `advanced.availability.${entry.availability}`,
      ...entry.kinds.map((kind) => `advanced.kind.${kind}`),
    ]),
    ...SHORTCUTS.flatMap((shortcut) => shortcut.keys.map((key) => `advanced.key.${key.toLowerCase()}`)),
    ...Object.values(UI_CONFIG.alertStyles).map((style) => style.labelKey),
    ...USER_ERROR_KEYS.map((key) => `error.${key}`),
    ...RECORDING_MODES.flatMap((mode) => [`record.mode.${mode}`, `record.mode.${mode}.hint`]),
    ...SESSION_STATUSES.flatMap((status) => [`session.status.${status}`, `session.note.${status}`]),
    ...SPEAKERS.map((speaker) => `session.speaker.${speaker}`),
    ...VIDEO_QUALITIES.map((quality) => `record.quality.${quality}`),
    ...TRANSCRIPTION_QUALITIES.map((quality) => `transcription.quality.${quality}`),
  ];
}

describe("translation keys", () => {
  it("are the same in the Spanish and English catalogs", () => {
    expect(Object.keys(CATALOGS.es ?? {}).sort()).toEqual(Object.keys(CATALOGS.en ?? {}).sort());
  });

  it("used as literals in the code exist in the catalogs", () => {
    const unknown = literalKeys().filter((literal) => !isKnown(literal));
    expect(unknown).toEqual([]);
  });

  it("built from lists of values exist in the catalogs", () => {
    const missing = builtKeys().filter((key) => !KEYS.has(key));
    expect(missing).toEqual([]);
  });
});
