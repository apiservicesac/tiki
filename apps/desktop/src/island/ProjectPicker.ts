import { ElementFactory } from "../dom/ElementFactory";
import { ICONS } from "../dom/iconLibrary";
import { TextMatcher } from "../dom/TextMatcher";

/** One choice of the picker: a project, or a special entry such as "All" or "No project". */
export interface ProjectPickerOption {
  /** The project's name, or `null` for the special entry. */
  value: string | null;
  label: string;
  color: string | null;
  /** Open tasks, shown on the right. */
  count?: number;
}

export interface ProjectPickerAction {
  label: string;
  onPress: () => void;
}

export interface ProjectPickerConfig {
  id: string;
  options: ProjectPickerOption[];
  selected: string | null;
  searchPlaceholder: string;
  emptyText: string;
  /** Extra entries under the list, such as "New project". */
  actions?: ProjectPickerAction[];
  onSelect: (value: string | null) => void;
}

/**
 * A button that opens a searchable list of projects. Works the same with three projects or
 * fifty: the list scrolls and the search box narrows it as the user types.
 */
export class ProjectPicker {
  readonly element: HTMLElement;
  private readonly trigger: HTMLButtonElement;
  private readonly popover: HTMLElement;
  private readonly search: HTMLInputElement;
  private readonly list: HTMLElement;
  private readonly footer: HTMLElement;
  private config: ProjectPickerConfig | null = null;
  private readonly closeOnOutsideClick = (event: PointerEvent) => {
    if (!this.element.contains(event.target as Node)) {
      this.close();
    }
  };

  constructor() {
    this.trigger = ElementFactory.create("button", { className: "picker-trigger", attributes: { type: "button" } });
    this.trigger.addEventListener("click", (event) => {
      event.stopPropagation();
      if (this.isOpen()) {
        this.close();
      } else {
        this.open();
      }
    });
    this.search = ElementFactory.create("input", {
      className: "settings-input picker-search",
      attributes: { type: "text", autocomplete: "off", spellcheck: "false" },
    });
    this.search.addEventListener("input", () => this.drawList());
    this.search.addEventListener("keydown", (event) => this.handleKey(event));
    this.list = ElementFactory.create("div", { className: "picker-list", attributes: { role: "listbox" } });
    this.footer = ElementFactory.create("div", { className: "picker-footer" });
    this.popover = ElementFactory.create("div", {
      className: "picker-popover",
      children: [this.search, this.list, this.footer],
    });
    this.popover.hidden = true;
    this.element = ElementFactory.create("div", { className: "picker", children: [this.trigger, this.popover] });
  }

  update(config: ProjectPickerConfig): void {
    this.config = config;
    this.trigger.id = config.id;
    this.search.placeholder = config.searchPlaceholder;
    this.search.setAttribute("aria-label", config.searchPlaceholder);
    const selected = config.options.find((option) => option.value === config.selected) ?? config.options[0];
    this.trigger.replaceChildren(
      ...(selected ? this.optionContent(selected) : []),
      ElementFactory.fromTrustedMarkup("picker-chevron", ICONS.chevron),
    );
    this.footer.replaceChildren(...(config.actions ?? []).map((action) => this.actionButton(action)));
    if (this.isOpen()) {
      this.drawList();
    }
  }

  isOpen(): boolean {
    return !this.popover.hidden;
  }

  close(): void {
    this.popover.hidden = true;
    this.element.classList.remove("picker-open");
    document.removeEventListener("pointerdown", this.closeOnOutsideClick, true);
  }

  private open(): void {
    this.search.value = "";
    this.popover.hidden = false;
    this.element.classList.add("picker-open");
    this.drawList();
    document.addEventListener("pointerdown", this.closeOnOutsideClick, true);
    queueMicrotask(() => this.search.focus());
  }

  private drawList(): void {
    const config = this.config;
    if (!config) {
      return;
    }
    const matches = this.matchingOptions(config);
    this.list.replaceChildren(
      ...(matches.length > 0
        ? matches.map((option) => this.optionButton(option, option.value === config.selected))
        : [ElementFactory.create("p", { className: "picker-empty", text: config.emptyText })]),
    );
  }

  /** Special entries always show; projects only when they match the search. */
  private matchingOptions(config: ProjectPickerConfig): ProjectPickerOption[] {
    const query = this.search.value;
    return config.options.filter((option) => option.value === null || TextMatcher.matches(option.label, query));
  }

  private handleKey(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      event.stopPropagation();
      this.close();
      this.trigger.focus();
    } else if (event.key === "Enter" && this.config) {
      event.preventDefault();
      const first = this.matchingOptions(this.config).find((option) => option.value !== null)
        ?? this.matchingOptions(this.config)[0];
      if (first) {
        this.choose(first.value);
      }
    }
  }

  private choose(value: string | null): void {
    this.close();
    this.config?.onSelect(value);
  }

  private optionButton(option: ProjectPickerOption, selected: boolean): HTMLElement {
    const button = ElementFactory.create("button", {
      className: selected ? "picker-option picker-option-on" : "picker-option",
      attributes: { type: "button", role: "option", "aria-selected": String(selected), "data-value": option.value ?? "" },
      children: this.optionContent(option),
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      this.choose(option.value);
    });
    return button;
  }

  private optionContent(option: ProjectPickerOption): HTMLElement[] {
    const dot = ElementFactory.create("span", { className: "picker-dot" });
    dot.style.background = option.color ?? "transparent";
    if (!option.color) {
      dot.classList.add("picker-dot-empty");
    }
    return [
      dot,
      ElementFactory.create("span", { className: "picker-label", text: option.label }),
      ...(option.count !== undefined && option.count > 0
        ? [ElementFactory.create("span", { className: "picker-count", text: String(option.count) })]
        : []),
    ];
  }

  private actionButton(action: ProjectPickerAction): HTMLElement {
    const button = ElementFactory.create("button", {
      className: "picker-action",
      text: action.label,
      attributes: { type: "button" },
    });
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      this.close();
      action.onPress();
    });
    return button;
  }
}
