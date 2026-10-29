import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AlertDelivery } from "../src/backend/contracts";
import type { IslandController } from "../src/island/IslandController";
import { AlertSound } from "../src/island/AlertSound";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";

const MEETING_ALERT = { kind: "meeting" as const, title: "Daily", body: "En 15 min", taskId: null, link: null };
const SILENT: AlertDelivery = { sound: false, windowsNotification: false };

describe("Tiki keeping quiet", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let controller: IslandController;

  const notch = () => root.querySelector<HTMLElement>(".island-notch");

  beforeEach(async () => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend();
    controller = await new TikiApp(root, backend).start();
  });

  afterEach(() => {
    controller.stop();
    vi.restoreAllMocks();
  });

  it("shows a silent alert in the notch without the chime", async () => {
    const chime = vi.spyOn(AlertSound.prototype, "play").mockImplementation(() => undefined);

    backend.deliverAlert(MEETING_ALERT, SILENT);
    await vi.waitFor(() => expect(notch()?.dataset.mode).toBe("alert"));
    expect(chime).not.toHaveBeenCalled();

    backend.deliverAlert(MEETING_ALERT);
    await vi.waitFor(() => expect(chime).toHaveBeenCalledOnce());
  });
});
