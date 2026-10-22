import { describe, expect, it } from "vitest";
import { CommandInterpreter, TikiAi, type AiProviderName, type InterpretedCapture, type ProxiedRequest, type ProxiedResponse } from "../../src/ai";
import cases from "./captureCases.json";

/**
 * Measures how well the AI reads real sentences, with the user's own key. It costs a few
 * cents per run, so it is not part of `make test`: run `make eval-ai` before a release.
 * Reads TIKI_EVAL_PROVIDER (anthropic or openai), TIKI_EVAL_MODEL and the provider's key
 * (ANTHROPIC_API_KEY or OPENAI_API_KEY) from the environment.
 */
const provider = (process.env.TIKI_EVAL_PROVIDER ?? "anthropic") as AiProviderName;
const key = provider === "anthropic" ? process.env.ANTHROPIC_API_KEY : process.env.OPENAI_API_KEY;

/** Plays the desktop shell: adds the real key and sends the request. */
async function sendWithKey(request: ProxiedRequest): Promise<ProxiedResponse> {
  const headers: Record<string, string> = Object.fromEntries(
    request.headers.filter(([name]) => !/^(authorization|x-api-key)$/i.test(name)),
  );
  if (provider === "anthropic") {
    headers["x-api-key"] = key ?? "";
  } else {
    headers.authorization = `Bearer ${key ?? ""}`;
  }
  const response = await fetch(request.url, { method: request.method, headers, body: request.body });
  const responseHeaders: Array<[string, string]> = [];
  response.headers.forEach((value, name) => responseHeaders.push([name, value]));
  return { status: response.status, headers: responseHeaders, body: await response.text() };
}

describe.skipIf(!key)(`capture eval with ${provider}`, () => {
  it("reads real sentences", { timeout: 300_000 }, async () => {
    const interpreter = new CommandInterpreter(new TikiAi(sendWithKey).model(provider, process.env.TIKI_EVAL_MODEL ?? null));
    let checks = 0;
    let passed = 0;
    for (const testCase of cases.cases) {
      const reply = await interpreter.read(testCase.sentence, {
        now: new Date(cases.now),
        language: "Spanish",
        advisor: { now: cases.now, openTasks: [], projects: cases.projects.map((name) => ({ name, notes: "" })) },
        lessons: [],
      });
      const expectedKind = "answer" in testCase ? "answer" : "capture";
      const expected: Record<string, unknown> = "expect" in testCase ? testCase.expect : {};
      const read: Partial<InterpretedCapture> = reply.kind === "capture" ? reply.item : {};
      const misses = [
        ...(reply.kind === expectedKind ? [] : [["intent", expectedKind, reply.kind]]),
        ...Object.entries(expected)
          .filter(([field, value]) => JSON.stringify(read[field as keyof InterpretedCapture]) !== JSON.stringify(value))
          .map(([field, value]) => [field, value, read[field as keyof InterpretedCapture]]),
      ];
      const fieldCount = Object.keys(expected).length + 1;
      checks += fieldCount;
      passed += fieldCount - misses.length;
      console.log(`${misses.length === 0 ? "✓" : "✗"} ${testCase.sentence}`);
      misses.forEach(([field, value, got]) =>
        console.log(`    ${String(field)}: expected ${JSON.stringify(value)}, got ${JSON.stringify(got)}`));
    }
    const score = passed / checks;
    console.log(`Capture eval: ${passed}/${checks} fields right (${Math.round(score * 100)}%)`);
    expect(score).toBeGreaterThanOrEqual(0.85);
  });
});
