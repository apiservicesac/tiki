import type { ProxiedRequest, ProxiedResponse } from "../../src/ai";

/** Stands in for the desktop shell: records what the SDKs send and answers with canned JSON. */
export class FakeShell {
  readonly requests: ProxiedRequest[] = [];

  constructor(private readonly answer: (request: ProxiedRequest) => unknown) {}

  readonly send = async (request: ProxiedRequest): Promise<ProxiedResponse> => {
    this.requests.push(request);
    return {
      status: 200,
      headers: [["content-type", "application/json"]],
      body: JSON.stringify(this.answer(request)),
    };
  };

  lastBody(): Record<string, unknown> {
    const body = this.requests.at(-1)?.body;
    return body ? JSON.parse(body) : {};
  }
}
