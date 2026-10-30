import { CoreDateTimeText } from "../backend/CoreDateTimeText";
import { ElementFactory } from "../dom/ElementFactory";
import type { Translator } from "../i18n/Translator";

/** One choice of a row with several: its value and its translated label. */
export interface SettingChoice<Value> {
  value: Value;
  label: string;
}

/** What one settings row says: its name and, under it, what it does. */
export interface SettingText {
  labelKey: string;
  hintKey: string;
}

/**
 * Builds the rows of a settings list: the name and a one-line explanation on the left, the
 * control on the right, so every option says what it does.
 */
export class SettingRows {
  constructor(private readonly translator: Translator) {}

  /** A titled block of rows. */
  section(titleKey: string, rows: HTMLElement[]): HTMLElement {
    return ElementFactory.create("section", {
      className: "settings-group",
      children: [
        ElementFactory.create("h3", { text: this.translator.text(titleKey) }),
        ElementFactory.create("div", { className: "settings-list", children: rows }),
      ],
    });
  }

  /** An on/off option; `inactive` dims it while an option it depends on is off. */
  switchRow(text: SettingText, enabled: boolean, onToggle: (enabled: boolean) => void, inactive = false): HTMLElement {
    const control = ElementFactory.textButton({
      className: "setting-switch",
      attributes: {
        role: "switch",
        "aria-checked": String(enabled),
        "aria-label": this.translator.text(text.labelKey),
        "data-setting": text.labelKey,
      },
    }, () => onToggle(!enabled));
    return this.row(text, [control], inactive);
  }

  /** One of a few choices, as pills; the chosen one is checked. */
  choiceRow<Value>(
    text: SettingText,
    choices: readonly SettingChoice<Value>[],
    chosen: Value,
    onChoose: (value: Value) => void,
  ): HTMLElement {
    const pills = choices.map((choice) => ElementFactory.textButton({
      className: "settings-option",
      text: choice.label,
      attributes: { role: "radio", "aria-checked": String(choice.value === chosen) },
    }, () => onChoose(choice.value)));
    return this.row(text, [ElementFactory.create("span", {
      className: "settings-options",
      attributes: { role: "radiogroup", "aria-label": this.translator.text(text.labelKey) },
      children: pills,
    })]);
  }

  /** A time of day; `onPick` receives it as the core writes it ("07:45:00"). */
  timeRow(text: SettingText, name: string, time: string, onPick: (time: string) => void): HTMLElement {
    return this.row(text, [this.timeInput(text.labelKey, name, time, onPick)]);
  }

  /** A range of the day, "from 22:00 to 07:00", as one row. */
  rangeRow(
    text: SettingText,
    from: { name: string; time: string; labelKey: string },
    until: { name: string; time: string; labelKey: string },
    onPick: (range: { from: string; until: string }) => void,
  ): HTMLElement {
    const fromInput = this.timeInput(from.labelKey, from.name, from.time, (time) => onPick({ from: time, until: until.time }));
    const untilInput = this.timeInput(until.labelKey, until.name, until.time, (time) => onPick({ from: from.time, until: time }));
    const range = ElementFactory.create("span", {
      className: "setting-range",
      children: [
        ElementFactory.create("span", { text: this.translator.text(from.labelKey) }),
        fromInput,
        ElementFactory.create("span", { text: this.translator.text(until.labelKey) }),
        untilInput,
      ],
    });
    return this.row(text, [range]);
  }

  private row(text: SettingText, controls: HTMLElement[], inactive = false): HTMLElement {
    return ElementFactory.create("div", {
      className: inactive ? "setting-row setting-row-inactive" : "setting-row",
      children: [
        ElementFactory.create("span", {
          className: "setting-row-text",
          children: [
            ElementFactory.create("strong", { text: this.translator.text(text.labelKey) }),
            ElementFactory.create("small", { text: this.translator.text(text.hintKey) }),
          ],
        }),
        ...controls,
      ],
    });
  }

  private timeInput(labelKey: string, name: string, time: string, onPick: (time: string) => void): HTMLInputElement {
    const input = ElementFactory.create("input", {
      className: "settings-input settings-time",
      attributes: { type: "time", id: `setting-${name}`, "aria-label": this.translator.text(labelKey) },
    });
    input.value = CoreDateTimeText.clock(time);
    // The clock icon is hidden; clicking anywhere on the field opens the picker instead.
    input.addEventListener("click", () => input.showPicker?.());
    input.addEventListener("change", () => {
      if (input.value !== "") {
        onPick(CoreDateTimeText.fromClock(input.value));
      }
    });
    return input;
  }
}
