import { describe, expect, it } from "vitest";
import { SessionNotesAdvisor, type NotesLine, type SessionNotesContext } from "../src/ai";
import { AI_CONFIG } from "../src/ai/aiConfig";

const context: SessionNotesContext = {
  title: "Revisión con Andeanvet",
  startedAt: "2026-10-21T10:00:00",
  language: "Spanish",
  projects: [{ name: "Andeanvet", notes: "Contacto: Lucía." }],
};

const written = {
  summary: "  Se revisó la facturación.  ",
  keyPoints: ["Octubre cerrado", " "],
  decisions: ["Enviar el informe mañana"],
  tasks: [
    { title: "Enviar el informe", dueDate: "2026-10-22" },
    { title: "Llamar a Lucía", dueDate: "el viernes" },
    { title: "  ", dueDate: null },
  ],
};

/** A model that answers every call with `answer` and keeps what it was asked. */
function recordingModel(answer: object) {
  const inputs: string[] = [];
  const instructions: string[] = [];
  return {
    inputs,
    instructions,
    model: {
      extract: async (task: { input: string; instructions: string }) => {
        inputs.push(task.input);
        instructions.push(task.instructions);
        return answer;
      },
    } as never,
  };
}

const lines: NotesLine[] = [
  { speaker: "others", startMs: 0, text: "Revisemos la facturación de octubre" },
  { speaker: "you", startMs: 65_000, text: "Mañana envío el informe" },
];

describe("SessionNotesAdvisor", () => {
  it("reads the transcript with each line's time and who said it", async () => {
    const reader = recordingModel(written);
    await new SessionNotesAdvisor(reader.model).write(lines, context);

    expect(reader.inputs).toHaveLength(1);
    expect(reader.inputs[0]).toContain("[00:00] Others: Revisemos la facturación de octubre");
    expect(reader.inputs[0]).toContain("[01:05] You (the user): Mañana envío el informe");
    expect(reader.inputs[0]).toContain("started 2026-10-21T10:00:00");
    expect(reader.instructions[0]).toContain("Contacto: Lucía.");
  });

  it("keeps clean notes and only due dates written as dates", async () => {
    const notes = await new SessionNotesAdvisor(recordingModel(written).model).write(lines, context);
    expect(notes).toEqual({
      summary: "Se revisó la facturación.",
      keyPoints: ["Octubre cerrado"],
      decisions: ["Enviar el informe mañana"],
      tasks: [
        { title: "Enviar el informe", dueDate: "2026-10-22" },
        { title: "Llamar a Lucía", dueDate: null },
      ],
    });
  });

  it("reads a long transcript in parts and joins their notes", async () => {
    const reader = recordingModel(written);
    const longLine = "x".repeat(AI_CONFIG.sessionNotesPartCharacters / 2);
    const long: NotesLine[] = Array.from({ length: 5 }, (_, index) => ({ speaker: "others", startMs: index * 1_000, text: longLine }));

    await new SessionNotesAdvisor(reader.model).write(long, context);

    const parts = reader.inputs.filter((input) => input.includes("Transcript:"));
    expect(parts.length).toBeGreaterThan(1);
    expect(parts[0]).toContain(`part 1 of ${parts.length}`);
    expect(reader.inputs).toHaveLength(parts.length + 1);
    expect(JSON.parse(reader.inputs.at(-1) ?? "[]")).toHaveLength(parts.length);
  });
});
