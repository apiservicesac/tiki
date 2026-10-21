import type { SendThroughShell } from "./contracts";

/**
 * The `fetch` given to the official SDKs. Instead of going to the network, each request
 * goes to the desktop shell, which adds the API key and sends it to the provider.
 */
export class ShellFetch {
  /** HTTP statuses whose responses must not carry a body. */
  private static readonly bodilessStatuses = new Set([204, 205, 304]);

  constructor(private readonly sendThroughShell: SendThroughShell) {}

  readonly fetch = async (input: string | URL | Request, init?: RequestInit): Promise<Response> => {
    const request = new Request(input, init);
    const hasBody = request.method !== "GET" && request.method !== "HEAD";
    const answer = await this.sendThroughShell({
      url: request.url,
      method: request.method,
      headers: [...request.headers.entries()],
      body: hasBody ? await request.text() : null,
    });
    const body = ShellFetch.bodilessStatuses.has(answer.status) ? null : answer.body;
    return new Response(body, { status: answer.status, headers: answer.headers });
  };
}
