import { beforeEach, describe, expect, it, vi } from "vitest";
import { AdvancedSettingsApp } from "../src/advanced/AdvancedSettingsApp";
import type { WindowControls } from "../src/advanced/WindowControls";
import { SHELL_EVENTS } from "../src/config/uiConfig";
import { FakeBackend } from "./support/FakeBackend";

class FakeWindowControls implements WindowControls {
  readonly calls: string[] = [];

  async minimize(): Promise<void> {
    this.calls.push("minimize");
  }

  async toggleMaximize(): Promise<void> {
    this.calls.push("maximize");
  }

  async close(): Promise<void> {
    this.calls.push("close");
  }
}

describe("AdvancedSettingsApp", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let windowControls: FakeWindowControls;
  let app: AdvancedSettingsApp;

  const startOn = async (section: string) => {
    app = new AdvancedSettingsApp(root, backend, windowControls, section);
    await app.start();
  };

  beforeEach(() => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend();
    windowControls = new FakeWindowControls();
  });

  it("opens on the requested section and moves between sections from the menu", async () => {
    await startOn("ai");
    expect(root.querySelector("h1")?.textContent).toBe("Asistente de IA");

    root.querySelector<HTMLButtonElement>('[data-section="shortcuts"]')?.click();
    expect(app.currentSection).toBe("shortcuts");
    expect(root.querySelectorAll(".shortcut-row")).toHaveLength(3);
  });

  it("falls back to General for an unknown section", async () => {
    await startOn("nope");
    expect(app.currentSection).toBe("general");
  });

  it("drives its own window from the title bar", async () => {
    await startOn("general");
    root.querySelectorAll<HTMLButtonElement>(".advanced-window-buttons button").forEach((button) => button.click());
    expect(windowControls.calls).toEqual(["minimize", "maximize", "close"]);
  });

  it("toggles start with Windows, shows the version and installs an update", async () => {
    await startOn("general");
    root.querySelector<HTMLButtonElement>('[data-setting="settings.autostart"]')?.click();
    await vi.waitFor(() => expect(backend.autostart).toBe(true));
    await vi.waitFor(() => expect(root.querySelector(".settings-updates")?.textContent).toContain("Versión 0.2.0"));

    backend.availableUpdate = { version: "0.2.1", notes: null };
    root.querySelector<HTMLButtonElement>("#update-action")?.click();
    await vi.waitFor(() => expect(root.querySelector("#update-action")?.textContent).toBe("Instalar 0.2.1"));
    root.querySelector<HTMLButtonElement>("#update-action")?.click();
    await vi.waitFor(() => expect(backend.installedUpdates).toBe(1));
  });

  it("saves a pasted key, lists the models and removes the key", async () => {
    await startOn("ai");
    const input = root.querySelector<HTMLInputElement>("#ai-key-input");
    if (input) {
      input.value = "sk-new";
    }
    root.querySelector<HTMLButtonElement>(".settings-ai .settings-button")?.click();
    await vi.waitFor(() => expect(root.querySelector(".settings-connected")).not.toBeNull());
    expect(backend.aiKeys.get("anthropic")).toBe("sk-new");

    const models = [...root.querySelectorAll<HTMLOptionElement>("#ai-model-select option")].map((option) => option.value);
    expect(models).toContain("claude-opus-5-5");

    root.querySelector<HTMLButtonElement>(".settings-ai .settings-button-quiet")?.click();
    await vi.waitFor(() => expect(root.querySelector("#ai-key-input")).not.toBeNull());
    expect(backend.aiKeys.has("anthropic")).toBe(false);
  });

  it("switches provider and keeps the island in sync through the shell", async () => {
    await startOn("ai");
    root.querySelectorAll<HTMLButtonElement>(".settings-ai .settings-option")[1]?.click();
    await vi.waitFor(() => expect(backend.savedPreferences.aiProvider).toBe("openai"));
  });

  it("follows a language change made in the island", async () => {
    await startOn("plugins");
    backend.savedPreferences = { ...backend.savedPreferences, locale: "english" };
    backend.emit(SHELL_EVENTS.preferencesChanged);
    await vi.waitFor(() => expect(root.querySelector("h1")?.textContent).toBe("Plugins and sources"));
    // The next plugin gets a card; the ones further away are named in one line.
    expect(root.querySelectorAll(".catalog-card")).toHaveLength(1);
    expect(root.querySelector(".catalog-later")?.textContent).toBe("Later: Slack, Jira, GitHub, Zendesk, and PostgreSQL.");
  });

  it("connects and disconnects calendar accounts", async () => {
    await startOn("accounts");
    await vi.waitFor(() => expect(root.querySelector('[data-provider="google"]')).not.toBeNull());
    expect(root.querySelector(".settings-calendars")?.textContent).toContain("Aún no hay ninguno.");

    root.querySelector<HTMLButtonElement>('[data-provider="google"]')?.click();
    await vi.waitFor(() => expect(root.querySelector(".calendar-account")).not.toBeNull());
    expect(root.querySelector(".calendar-account .settings-connected")?.firstChild?.textContent).toBe("Ana Pérez");
    expect(root.querySelector(".calendar-account small")?.textContent).toBe("ana@google.test · Google Calendar");
    expect(root.querySelector<HTMLElement>(".calendar-account .account-service")?.dataset.service).toBe("google");

    root.querySelector<HTMLButtonElement>('[data-account="1"]')?.click();
    await vi.waitFor(() => expect(root.querySelector(".calendar-account")).toBeNull());
  });

  it("disables the providers this build cannot connect", async () => {
    backend.calendar = { accounts: [], available: ["google"] };
    await startOn("accounts");
    await vi.waitFor(() => expect(root.querySelector<HTMLButtonElement>('[data-provider="microsoft"]')?.disabled).toBe(true));
    expect(root.querySelector<HTMLButtonElement>('[data-provider="google"]')?.disabled).toBe(false);
  });

  it("chooses the transcription model, the automatic notes and how long media is kept", async () => {
    await startOn("sessions");
    const option = (label: string) => [...root.querySelectorAll<HTMLButtonElement>(".settings-option")]
      .find((button) => button.textContent === label);
    option("Precisa")?.click();
    await vi.waitFor(() => expect(backend.savedPreferences.transcriptionQuality).toBe("precise"));
    root.querySelector<HTMLButtonElement>('[data-setting="advanced.sessions.notes_automatic"]')?.click();
    await vi.waitFor(() => expect(backend.savedPreferences.sessionNotesAutomatic).toBe(false));
    option("30 días")?.click();
    await vi.waitFor(() => expect(backend.savedPreferences.sessionMediaDays).toBe(30));
  });
});
