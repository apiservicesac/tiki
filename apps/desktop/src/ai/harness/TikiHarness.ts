import { AiDates } from "../AiDates";
import { HarnessPieces, type HarnessLanguage, type HarnessLesson, type HarnessProject } from "./HarnessPieces";

/** Builds the instructions for each of Tiki's jobs from the harness pieces. */
export class TikiHarness {
  /** The single box: capture an item, or answer and propose actions. */
  static command(
    language: HarnessLanguage,
    now: Date,
    projectNames: readonly string[],
    projects: readonly HarnessProject[],
    lessons: readonly HarnessLesson[],
  ): string {
    return TikiHarness.join([
      HarnessPieces.persona(language),
      HarnessPieces.guardrails(),
      HarnessPieces.commandSkill(),
      HarnessPieces.captureSkill(AiDates.describeNow(now), projectNames),
      HarnessPieces.answerSkill(),
      HarnessPieces.toolGuide(),
      HarnessPieces.memory(projects, lessons),
    ]);
  }

  static brief(language: HarnessLanguage, projects: readonly HarnessProject[]): string {
    return TikiHarness.join([
      HarnessPieces.persona(language),
      HarnessPieces.guardrails(),
      HarnessPieces.briefSkill(),
      HarnessPieces.memory(projects, []),
    ]);
  }

  private static join(pieces: string[]): string {
    return pieces.filter((piece) => piece !== "").join("\n\n");
  }
}
