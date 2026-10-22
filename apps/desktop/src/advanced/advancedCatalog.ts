/**
 * What the advanced settings list: sections and plugins. Plugins are shown with their real
 * availability; none of them is connected yet.
 */
export const ADVANCED_SECTIONS = Object.freeze(["general", "ai", "plugins", "accounts", "shortcuts"] as const);
export type AdvancedSection = (typeof ADVANCED_SECTIONS)[number];

export type PluginKind = "ticket_source" | "client_instance";
export type Availability = "next" | "later";

export interface CatalogEntry {
  id: string;
  name: string;
  kinds: readonly PluginKind[];
  availability: Availability;
}

export const PLUGIN_CATALOG: readonly CatalogEntry[] = Object.freeze([
  { id: "odoo", name: "Odoo", kinds: ["ticket_source", "client_instance"], availability: "next" },
  { id: "slack", name: "Slack", kinds: ["ticket_source"], availability: "later" },
  { id: "jira", name: "Jira", kinds: ["ticket_source"], availability: "later" },
  { id: "github", name: "GitHub", kinds: ["ticket_source", "client_instance"], availability: "later" },
  { id: "zendesk", name: "Zendesk", kinds: ["ticket_source"], availability: "later" },
  { id: "postgresql", name: "PostgreSQL", kinds: ["client_instance"], availability: "later" },
]);

/** Must match `DesktopConfig::SHORTCUT_*` in the shell. */
export const SHORTCUTS = Object.freeze([
  { keys: ["Ctrl", "Alt", "Space"], labelKey: "advanced.shortcuts.capture" },
  { keys: ["Ctrl", "Alt", "N"], labelKey: "advanced.shortcuts.toggle" },
]);
