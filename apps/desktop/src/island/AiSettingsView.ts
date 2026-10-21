import { TikiAi, type AiModelOption, type AiProviderName } from "../ai";
import { AI_PROVIDERS, type UserPreferences } from "../backend/contracts";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

/** What the AI section shows: which providers have a key, and the models to choose from. */
export interface AiSettingsState {
  configured: Record<AiProviderName, boolean>;
  models: AiModelOption[];
}

export interface AiSettingsHandlers {
  onChangePreferences: (preferences: UserPreferences) => void;
  onSaveKey: (provider: AiProviderName, apiKey: string) => void;
  onRemoveKey: (provider: AiProviderName) => void;
}

/**
 * Settings for the AI assistant: provider, API key and model. The key box is kept between
 * redraws, so a background refresh never wipes a key the user is pasting.
 */
export class AiSettingsView {
  private keyInput: { provider: AiProviderName; element: HTMLInputElement } | null = null;

  constructor(private readonly translator: Translator) {}

  render(preferences: UserPreferences, state: AiSettingsState, handlers: AiSettingsHandlers): HTMLElement {
    const provider = preferences.aiProvider;
    return ElementFactory.create("section", {
      className: "settings-group settings-ai",
      children: [
        ElementFactory.create("h3", { text: this.translator.text("settings.ai") }),
        this.providerChoice(preferences, handlers),
        state.configured[provider] ? this.connectedKey(provider, handlers) : this.keyForm(provider, handlers),
        this.modelChoice(preferences, state.models, handlers),
        ElementFactory.create("p", { className: "settings-note", text: this.translator.text("settings.ai.privacy") }),
      ],
    });
  }

  private providerChoice(preferences: UserPreferences, handlers: AiSettingsHandlers): HTMLElement {
    const options = AI_PROVIDERS.map((provider) => {
      const button = ElementFactory.create("button", {
        className: "settings-option",
        text: this.translator.text(`provider.${provider}`),
        attributes: { type: "button", role: "radio", "aria-checked": String(preferences.aiProvider === provider) },
      });
      button.addEventListener("click", (event) => {
        event.stopPropagation();
        handlers.onChangePreferences({ ...preferences, aiProvider: provider, aiModel: null });
      });
      return button;
    });
    return ElementFactory.create("div", { className: "settings-options", attributes: { role: "radiogroup" }, children: options });
  }

  private keyForm(provider: AiProviderName, handlers: AiSettingsHandlers): HTMLElement {
    const input = this.keyInputFor(provider);
    const save = () => {
      const apiKey = input.value.trim();
      if (apiKey !== "") {
        input.value = "";
        handlers.onSaveKey(provider, apiKey);
      }
    };
    input.onkeydown = (event) => {
      if (event.key === "Enter") {
        save();
      }
    };
    const button = ElementFactory.create("button", {
      className: "settings-button",
      text: this.translator.text("settings.ai.save_key"),
      attributes: { type: "button" },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      save();
    });
    return ElementFactory.create("div", { className: "settings-row", children: [input, button] });
  }

  private keyInputFor(provider: AiProviderName): HTMLInputElement {
    if (this.keyInput?.provider !== provider) {
      const element = ElementFactory.create("input", {
        className: "settings-input",
        attributes: {
          type: "password",
          id: "ai-key-input",
          autocomplete: "off",
          spellcheck: "false",
          placeholder: this.translator.text("settings.ai.key_placeholder"),
          "aria-label": this.translator.text("settings.ai.key"),
        },
      });
      this.keyInput = { provider, element };
    }
    return this.keyInput.element;
  }

  private connectedKey(provider: AiProviderName, handlers: AiSettingsHandlers): HTMLElement {
    const remove = ElementFactory.create("button", {
      className: "settings-button settings-button-quiet",
      text: this.translator.text("settings.ai.remove_key"),
      attributes: { type: "button" },
    });
    remove.addEventListener("click", (event) => {
      event.stopPropagation();
      handlers.onRemoveKey(provider);
    });
    return ElementFactory.create("div", {
      className: "settings-row",
      children: [
        ElementFactory.create("span", { className: "settings-connected", text: this.translator.text("settings.ai.connected") }),
        remove,
      ],
    });
  }

  private modelChoice(preferences: UserPreferences, models: AiModelOption[], handlers: AiSettingsHandlers): HTMLElement {
    const selected = preferences.aiModel ?? TikiAi.defaultModel(preferences.aiProvider);
    const choices = models.some((model) => model.id === selected) ? models : [{ id: selected, label: selected }, ...models];
    const select = ElementFactory.create("select", {
      className: "settings-select",
      attributes: { id: "ai-model-select", "aria-label": this.translator.text("settings.ai.model") },
      children: choices.map((model) =>
        ElementFactory.create("option", { text: model.label, attributes: { value: model.id } })),
    });
    select.value = selected;
    select.addEventListener("change", () => handlers.onChangePreferences({ ...preferences, aiModel: select.value }));
    return ElementFactory.create("label", {
      className: "settings-row",
      children: [ElementFactory.create("span", { text: this.translator.text("settings.ai.model") }), select],
    });
  }
}
