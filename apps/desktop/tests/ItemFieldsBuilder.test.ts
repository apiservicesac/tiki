import { describe, expect, it } from "vitest";
import { Translator } from "../src/i18n/Translator";
import { ItemFieldsBuilder } from "../src/island/ItemFieldsBuilder";
import { ProjectPicker } from "../src/island/ProjectPicker";

describe("ItemFieldsBuilder", () => {
  const builder = new ItemFieldsBuilder(new Translator("en"));

  it("lists every priority with its translation and selects one", () => {
    const priority = builder.priority("priority", "high");
    expect([...priority.options].map((option) => option.textContent)).toEqual(["Low", "Normal", "High", "Urgent"]);
    expect(priority.value).toBe("high");
  });

  it("shows a reminder's moment in minutes", () => {
    expect(builder.remindAt("remind", "2026-10-29T09:30:00").value).toBe("2026-10-29T09:30");
    expect(builder.remindAt("remind", null).value).toBe("");
  });

  it("keeps the project the user picks", () => {
    const picker = new ProjectPicker();
    const choice = { value: null as string | null };
    builder.drawProjectPicker(picker, "project", [{ projectId: 1, name: "Acme", color: "#2F6BD8", notes: "", openCount: 2, pressingCount: 0 }], choice);
    picker.element.querySelector<HTMLButtonElement>(".picker-trigger")?.click();
    picker.element.querySelector<HTMLButtonElement>('[data-value="Acme"]')?.click();
    expect(choice.value).toBe("Acme");
  });

  it("puts a wide field across the form and the picker in a plain block", () => {
    const control = document.createElement("input");
    expect(builder.field("edit.field.notes", control, { wide: true }).className).toBe("edit-field edit-field-wide");
    expect(builder.field("capture.field.project", control, { block: true }).tagName).toBe("DIV");
  });
});
