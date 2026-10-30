import { describe, expect, it } from "vitest";
import type { AccountIdentity } from "../src/backend/contracts";
import { AccountBadge } from "../src/dom/AccountBadge";

const ANA: AccountIdentity = { provider: "google", email: "ana@acme.com", displayName: "Ana Pérez", avatar: "data:image/jpeg;base64,/9j/4AAQ" };

describe("AccountBadge", () => {
  it("shows the account's picture with the service's icon", () => {
    const badge = AccountBadge.render(ANA, "google_meet");
    expect(badge.querySelector("img")?.getAttribute("src")).toBe(ANA.avatar);
    expect(badge.querySelector<HTMLElement>(".account-service")?.dataset.service).toBe("google_meet");
  });

  it("shows only the service's icon while there is no picture", () => {
    const badge = AccountBadge.render({ ...ANA, avatar: null }, "microsoft");
    expect(badge.querySelector("img")).toBeNull();
    expect(badge.querySelector<HTMLElement>(".account-service")?.dataset.service).toBe("microsoft");
  });
});
