import { describe, expect, it } from "vitest";
import { CommandInterpreter, TikiAi, type CommandContext } from "../src/ai";
import { AI_CONFIG } from "../src/ai/aiConfig";
import { FakeShell } from "./support/FakeShell";

const item = {
  kind: "task",
  title: "Revisar acceso de facturas",
  projectName: "andeanvet",
  dueDate: "2026-10-23",
  remindAt: "2026-10-23T09:00",
  repeat: "never",
  waitingFor: null,
  tags: ["#Accesos"],
  priority: "high",
};

const captureReply = { intent: "capture", item, answer: null, actions: [] };

const context: CommandContext = {
  now: new Date(2026, 9, 21, 17, 30),
  language: "Spanish",
  advisor: {
    now: "2026-10-21T17:30:00",
    openTasks: [
      { taskId: 7, kind: "task", title: "Acta CAPEX", projectName: "Altamira", tags: [], status: "to_do", priority: "normal", dueDate: null, remindAt: null, waitingFor: null, waitingDays: null } as never,
    ],
    projects: [{ name: "Andeanvet", notes: "Contacto: Lucía." }],
  },
  lessons: [{ sentence: "revisar almendra", understood: { projectName: null }, corrected: { projectName: "Andeanvet" } }],
};

const fakeModel = (answer: object) => {
  const seen = { instructions: "" };
  return {
    seen,
    model: {
      extract: async (task: { instructions: string }) => {
        seen.instructions = task.instructions;
        return answer;
      },
    } as never,
  };
};

describe("TikiAi with Anthropic", () => {
  const claudeAnswer = () => ({
    id: "msg_test",
    type: "message",
    role: "assistant",
    model: "claude-opus-5-5",
    content: [{ type: "text", text: JSON.stringify(captureReply) }],
    stop_reason: "end_turn",
    stop_sequence: null,
    usage: { input_tokens: 10, output_tokens: 10 },
  });

  it("sends a structured request through the shell, without the real key", async () => {
    const shell = new FakeShell(claudeAnswer);
    const reply = await new CommandInterpreter(new TikiAi(shell.send).model("anthropic", null))
      .read("revisar acceso facturas andeanvet viernes, importante", context);

    const request = shell.requests[0];
    expect(request?.url.startsWith("https://api.anthropic.com/v1/messages")).toBe(true);
    const authorization = request?.headers.find(([name]) => /^(authorization|x-api-key)$/i.test(name));
    expect(authorization?.[1]).toContain(AI_CONFIG.placeholderApiKey);
    const body = shell.lastBody();
    expect(body.model).toBe("claude-opus-5-5");
    expect(body.fallbacks).toBe("default");
    expect(body.output_config).toMatchObject({ effort: "low", format: { type: "json_schema" } });
    expect(JSON.stringify(body)).toContain("revisar almendra");
    expect(JSON.stringify(body)).toContain("Contacto: Lucía.");

    expect(reply).toEqual({
      kind: "capture",
      item: { ...item, projectName: "Andeanvet", remindAt: "2026-10-23T09:00:00", tags: ["accesos"] },
    });
  });

  it("rejects an answer the model refused", async () => {
    const shell = new FakeShell(() => ({ ...claudeAnswer(), content: [], stop_reason: "refusal" }));
    const model = new TikiAi(shell.send).model("anthropic", null);

    await expect(new CommandInterpreter(model).read("x", context)).rejects.toMatchObject({ reason: "refused" });
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
        content: [{ type: "output_text", text: JSON.stringify(captureReply), annotations: [] }],
      },
    ],
    usage: { input_tokens: 10, output_tokens: 10, total_tokens: 20 },
  });

  it("uses the Responses API with a JSON schema and reads the parsed answer", async () => {
    const shell = new FakeShell(openAiAnswer);
    const reply = await new CommandInterpreter(new TikiAi(shell.send).model("openai", null)).read("revisar acceso", context);

    expect(shell.requests[0]?.url).toBe("https://api.openai.com/v1/responses");
    const body = shell.lastBody();
    expect(body.model).toBe("gpt-5.5");
    expect(body.text).toMatchObject({ format: { type: "json_schema", name: "tiki_command" } });
    expect(reply.kind === "capture" && reply.item.projectName).toBe("Andeanvet");
  });
});

describe("CommandInterpreter", () => {
  it("keeps a new project by name, drops malformed dates and keeps a reminder without moment as a task", async () => {
    const { model } = fakeModel({
      intent: "capture",
      item: { ...item, kind: "reminder", title: " Llamar ", projectName: " Constructora Sur ", dueDate: "viernes", remindAt: "mañana a las 9", repeat: "daily", tags: [] },
      answer: null,
      actions: [],
    });
    const reply = await new CommandInterpreter(model).read("llamar", context);

    expect(reply).toEqual({
      kind: "capture",
      item: {
        title: "Llamar",
        projectName: "Constructora Sur",
        dueDate: null,
        priority: "high",
        remindAt: null,
        kind: "task",
        tags: [],
        repeat: "never",
        waitingFor: null,
      },
    });
  });

  it("answers and keeps only the actions Tiki can apply", async () => {
    const { model, seen } = fakeModel({
      intent: "answer",
      item: null,
      answer: "Te propongo dos cambios.",
      actions: [
        { type: "set_due_date", summary: "Acta CAPEX vence el viernes", taskId: 7, title: null, projectName: null, dueDate: "2026-10-23", remindAt: null, status: null },
        { type: "move_task", summary: "Mover una tarea que no existe", taskId: 99, title: null, projectName: null, dueDate: null, remindAt: null, status: "done" },
        { type: "create_reminder", summary: "Recordarte llamar a Lucía", taskId: null, title: "Llamar a Lucía", projectName: "altamira", dueDate: null, remindAt: "2026-10-22T10:00", status: null },
      ],
    });
    const reply = await new CommandInterpreter(model).read("mueve el acta al viernes", context);

    expect(reply.kind).toBe("answer");
    if (reply.kind === "answer") {
      expect(reply.text).toBe("Te propongo dos cambios.");
      expect(reply.actions.map((action) => action.type)).toEqual(["set_due_date", "create_reminder"]);
      expect(reply.actions[1]).toMatchObject({ projectName: "Altamira", remindAt: "2026-10-22T10:00:00" });
    }
    expect(seen.instructions).toContain("You never change anything yourself");
    expect(seen.instructions).toContain("never invent one");
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
