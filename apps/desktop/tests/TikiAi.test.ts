import { describe, expect, it } from "vitest";
import { CaptureInterpreter, TikiAi } from "../src/ai";
import { AI_CONFIG } from "../src/ai/aiConfig";
import { FakeShell } from "./support/FakeShell";

const capturedTask = {
  title: "Revisar acceso de facturas",
  projectName: "andeanvet",
  dueDate: "2026-10-23",
  priority: "high",
  remindAt: "2026-10-23T09:00",
};

const context = { projectNames: ["Altamira", "Andeanvet"], today: new Date(2026, 9, 21) };

describe("TikiAi with Anthropic", () => {
  const claudeAnswer = () => ({
    id: "msg_test",
    type: "message",
    role: "assistant",
    model: "claude-opus-5-5",
    content: [{ type: "text", text: JSON.stringify(capturedTask) }],
    stop_reason: "end_turn",
    stop_sequence: null,
    usage: { input_tokens: 10, output_tokens: 10 },
  });

  it("sends a structured request through the shell, without the real key", async () => {
    const shell = new FakeShell(claudeAnswer);
    const interpreter = new CaptureInterpreter(new TikiAi(shell.send).model("anthropic", null));

    const capture = await interpreter.interpret("revisar acceso facturas andeanvet viernes, importante", context);

    const request = shell.requests[0];
    expect(request?.url.startsWith("https://api.anthropic.com/v1/messages")).toBe(true);
    const authorization = request?.headers.find(([name]) => /^(authorization|x-api-key)$/i.test(name));
    expect(authorization?.[1]).toContain(AI_CONFIG.placeholderApiKey);
    const body = shell.lastBody();
    expect(body.model).toBe("claude-opus-5-5");
    expect(body.fallbacks).toBe("default");
    expect(body.output_config).toMatchObject({ effort: "low", format: { type: "json_schema" } });

    expect(capture).toEqual({ ...capturedTask, projectName: "Andeanvet", remindAt: "2026-10-23T09:00:00" });
  });

  it("rejects an answer the model refused", async () => {
    const shell = new FakeShell(() => ({ ...claudeAnswer(), content: [], stop_reason: "refusal" }));
    const model = new TikiAi(shell.send).model("anthropic", null);

    await expect(new CaptureInterpreter(model).interpret("x", context)).rejects.toMatchObject({ reason: "refused" });
  });
});

describe("TikiAi with OpenAI", () => {
  const openAiAnswer = () => ({
    id: "resp_test",
    object: "response",
    created_at: 1792600000,
    status: "completed",
    model: "gpt-5.5",
    output: [
      {
        id: "msg_test",
        type: "message",
        role: "assistant",
        status: "completed",
        content: [{ type: "output_text", text: JSON.stringify(capturedTask), annotations: [] }],
      },
    ],
    usage: { input_tokens: 10, output_tokens: 10, total_tokens: 20 },
  });

  it("uses the Responses API with a JSON schema and reads the parsed answer", async () => {
    const shell = new FakeShell(openAiAnswer);
    const interpreter = new CaptureInterpreter(new TikiAi(shell.send).model("openai", null));

    const capture = await interpreter.interpret("revisar acceso facturas andeanvet viernes", context);

    expect(shell.requests[0]?.url).toBe("https://api.openai.com/v1/responses");
    const body = shell.lastBody();
    expect(body.model).toBe("gpt-5.5");
    expect(body.text).toMatchObject({ format: { type: "json_schema", name: "captured_task" } });
    expect(capture.projectName).toBe("Andeanvet");
  });
});

describe("CaptureInterpreter", () => {
  it("drops unknown projects and malformed dates", async () => {
    const model = {
      extract: async () => ({
        title: " Llamar ",
        projectName: "Desconocido",
        dueDate: "viernes",
        priority: "normal",
        remindAt: "mañana a las 9",
      }),
    };
    const capture = await new CaptureInterpreter(model as never).interpret("llamar", context);

    expect(capture).toEqual({ title: "Llamar", projectName: null, dueDate: null, priority: "normal", remindAt: null });
  });
});

describe("TikiAi model lists", () => {
  it("offers the curated list when no key is configured", async () => {
    const shell = new FakeShell(() => ({}));
    const models = await new TikiAi(shell.send).availableModels("openai", false);

    expect(models.map((model) => model.id)).toContain("gpt-5.5");
    expect(shell.requests).toHaveLength(0);
  });

  it("asks the provider for its live list when a key is configured", async () => {
    const shell = new FakeShell(() => ({
      data: [
        { id: "claude-opus-5-5", display_name: "Claude Opus 5.5", type: "model", created_at: "2026-09-01T00:00:00Z" },
        { id: "claude-haiku-5-5", display_name: "Claude Haiku 5.5", type: "model", created_at: "2026-09-01T00:00:00Z" },
      ],
      has_more: false,
      first_id: "claude-opus-5-5",
      last_id: "claude-haiku-5-5",
    }));
    const models = await new TikiAi(shell.send).availableModels("anthropic", true);

    expect(models).toEqual([
      { id: "claude-opus-5-5", label: "Claude Opus 5.5" },
      { id: "claude-haiku-5-5", label: "Claude Haiku 5.5" },
    ]);
  });

  it("falls back to the curated list when the provider cannot be reached", async () => {
    const failing = async () => {
      throw new Error("offline");
    };
    const models = await new TikiAi(failing).availableModels("anthropic", true);

    expect(models.map((model) => model.id)).toContain("claude-opus-5-5");
  });
});
