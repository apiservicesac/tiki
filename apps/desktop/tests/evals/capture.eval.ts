import { describe, expect, it } from "vitest";
import { CaptureInterpreter, TikiAi, type AiProviderName, type InterpretedCapture, type ProxiedRequest, type ProxiedResponse } from "../../src/ai";
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
    const interpreter = new CaptureInterpreter(new TikiAi(sendWithKey).model(provider, process.env.TIKI_EVAL_MODEL ?? null));
    let checks = 0;
    let passed = 0;
    for (const testCase of cases.cases) {
      const read = await interpreter.interpret(testCase.sentence, {
        projectNames: cases.projects,
        now: new Date(cases.now),
        lessons: [],
      });
      const misses = Object.entries(testCase.expect).filter(([field, expected]) =>
        JSON.stringify(read[field as keyof InterpretedCapture]) !== JSON.stringify(expected));
      checks += Object.keys(testCase.expect).length;
      passed += Object.keys(testCase.expect).length - misses.length;
      console.log(`${misses.length === 0 ? "✓" : "✗"} ${testCase.sentence}`);
      misses.forEach(([field, expected]) =>
        console.log(`    ${field}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(read[field as keyof InterpretedCapture])}`));
    }
    const score = passed / checks;
    console.log(`Capture eval: ${passed}/${checks} fields right (${Math.round(score * 100)}%)`);
    expect(score).toBeGreaterThanOrEqual(0.85);
  });
});
