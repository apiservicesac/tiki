import { AiDates } from "../AiDates";
import { HarnessPieces, type HarnessLanguage, type HarnessLesson, type HarnessProject } from "./HarnessPieces";

/** Builds the instructions for each of Tiki's jobs from the harness pieces. */
export class TikiHarness {
  static capture(now: Date, projectNames: readonly string[], lessons: readonly HarnessLesson[]): string {
    return TikiHarness.join([
      HarnessPieces.guardrails(),
      HarnessPieces.captureSkill(AiDates.describeNow(now), projectNames),
      HarnessPieces.memory([], lessons),
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

  static answer(language: HarnessLanguage, projects: readonly HarnessProject[]): string {
    return TikiHarness.join([
      HarnessPieces.persona(language),
      HarnessPieces.guardrails(),
      HarnessPieces.answerSkill(),
      HarnessPieces.toolGuide(),
      HarnessPieces.memory(projects, []),
    ]);
  }

  private static join(pieces: string[]): string {
    return pieces.filter((piece) => piece !== "").join("\n\n");
  }
}
