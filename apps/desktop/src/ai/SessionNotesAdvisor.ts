import { z } from "zod";
import { DurationText } from "../dom/DurationText";
import { AI_CONFIG } from "./aiConfig";
import { AiDates } from "./AiDates";
import type { StructuredModel } from "./contracts";
import type { HarnessLanguage, HarnessProject } from "./harness/HarnessPieces";
import { TikiHarness } from "./harness/TikiHarness";

/** One transcript line as the advisor reads it (see `TranscriptSegment`). */
export interface NotesLine {
  speaker: "you" | "others";
  startMs: number;
  text: string;
}

/** A task the AI proposes from the session; the user applies it with one click. */
export interface ProposedSessionTask {
  title: string;
  /** ISO date, for example `2026-10-23`. */
  dueDate: string | null;
}

/** What the AI writes about a session. */
export interface WrittenSessionNotes {
  summary: string;
  keyPoints: string[];
  decisions: string[];
  tasks: ProposedSessionTask[];
}

export interface SessionNotesContext {
  title: string;
  /** When the session started, for example `2026-10-21T10:00:00`, to resolve "tomorrow". */
  startedAt: string;
  language: HarnessLanguage;
  projects: readonly HarnessProject[];
}

/**
 * Writes a session's notes from its transcript: a summary, key points, decisions and the
 * tasks it leaves. A long transcript is read in parts, each summarized on its own, and the
 * parts' notes are then joined into one.
 */
export class SessionNotesAdvisor {
  private static readonly schema = z.object({
    summary: z.string(),
    keyPoints: z.array(z.string()),
    decisions: z.array(z.string()),
    tasks: z.array(z.object({ title: z.string(), dueDate: z.string().nullable() })),
  });

  /** How the transcript names each speaker for the model. */
  private static readonly SPEAKER_NAMES: Readonly<Record<NotesLine["speaker"], string>> = Object.freeze({
    you: "You (the user)",
    others: "Others",
  });

  constructor(private readonly model: StructuredModel) {}

  async write(lines: readonly NotesLine[], context: SessionNotesContext): Promise<WrittenSessionNotes> {
    const parts = SessionNotesAdvisor.split(lines.map((line) => SessionNotesAdvisor.lineText(line)));
    const partNotes = await Promise.all(parts.map((part, index) => this.extract(
      TikiHarness.sessionNotes(context.language, context.projects),
      SessionNotesAdvisor.partInput(context, part, index, parts.length),
    )));
    const notes = partNotes.length === 1 && partNotes[0]
      ? partNotes[0]
      : await this.extract(TikiHarness.joinSessionNotes(context.language), JSON.stringify(partNotes));
    return SessionNotesAdvisor.clean(notes);
  }

  private extract(instructions: string, input: string): Promise<z.infer<typeof SessionNotesAdvisor.schema>> {
    return this.model.extract({ instructions, input, schema: SessionNotesAdvisor.schema, schemaName: "session_notes" });
  }

  /** "[12:05] Others: Let's look at October's invoices". */
  private static lineText(line: NotesLine): string {
    return `[${DurationText.clock(line.startMs)}] ${SessionNotesAdvisor.SPEAKER_NAMES[line.speaker]}: ${line.text}`;
  }

  /** The transcript in parts of at most `partCharacters`, never cutting a line. */
  private static split(lines: readonly string[]): string[] {
    const parts: string[][] = [[]];
    let length = 0;
    for (const line of lines) {
      const current = parts.at(-1) ?? [];
      if (length + line.length > AI_CONFIG.sessionNotesPartCharacters && current.length > 0) {
        parts.push([line]);
        length = line.length;
      } else {
        current.push(line);
        length += line.length;
      }
    }
    return parts.map((part) => part.join("\n"));
  }

  private static partInput(context: SessionNotesContext, part: string, index: number, total: number): string {
    const header = `Session "${context.title}", started ${context.startedAt}.`;
    const position = total > 1 ? ` This is part ${index + 1} of ${total} of the transcript.` : "";
    return `${header}${position}\nTranscript:\n${part}`;
  }

  /** Trimmed text, no empty lines, and only due dates the model wrote as YYYY-MM-DD. */
  private static clean(notes: z.infer<typeof SessionNotesAdvisor.schema>): WrittenSessionNotes {
    const lines = (texts: string[]) => texts.map((text) => text.trim()).filter((text) => text !== "");
    return {
      summary: notes.summary.trim(),
      keyPoints: lines(notes.keyPoints),
      decisions: lines(notes.decisions),
      tasks: notes.tasks
        .map((task) => ({ title: task.title.trim(), dueDate: AiDates.date(task.dueDate) }))
        .filter((task) => task.title !== ""),
    };
  }
}
