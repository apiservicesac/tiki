import { afterEach, beforeEach, describe, expect, it } from "vitest";
import type { IslandSnapshot, SystemStats } from "../src/backend/contracts";
import { Translator } from "../src/i18n/Translator";
import type { IslandController } from "../src/island/IslandController";
import { SystemStatsText } from "../src/island/SystemStatsText";
import { TikiApp } from "../src/TikiApp";
import { FakeBackend } from "./support/FakeBackend";

const CALM: SystemStats = { cpuPercent: 23.4, memoryPercent: 61, temperatureCelsius: 54.2, downloadBytesPerSecond: 1_234_000 };

describe("SystemStatsText", () => {
  const text = new SystemStatsText(new Translator("es"));

  it("writes short figures in the language's number format", () => {
    expect(text.figures(CALM).map((figure) => figure.text)).toEqual(["CPU 23%", "RAM 61%", "54 °C", "↓ 1,2 MB/s"]);
  });

  it("leaves the temperature out where there is no sensor, and shows slow speeds in kB/s", () => {
    const figures = text.figures({ ...CALM, temperatureCelsius: null, downloadBytesPerSecond: 820_000 });
    expect(figures.map((figure) => figure.kind)).toEqual(["cpu", "memory", "download"]);
    expect(figures.at(-1)?.text).toBe("↓ 820 kB/s");
  });

  it("marks the figures that run high", () => {
    const busy = text.figures({ ...CALM, cpuPercent: 97, memoryPercent: 93, temperatureCelsius: 91 });
    expect(busy.filter((figure) => figure.high).map((figure) => figure.kind)).toEqual(["cpu", "memory", "temperature"]);
  });
});

describe("The resting notch with nothing pending", () => {
  let root: HTMLElement;
  let backend: FakeBackend;
  let controller: IslandController;

  const calmDay = (): IslandSnapshot => ({ ...FakeBackend.sampleSnapshot(), attentionCount: 0, mood: "calm" });

  beforeEach(async () => {
    root = document.createElement("div");
    document.body.replaceChildren(root);
    backend = new FakeBackend(calmDay());
    controller = await new TikiApp(root, backend).start();
  });

  afterEach(() => controller.stop());

  it("shows the computer's figures beside the face", () => {
    backend.deliverSystemStats(CALM);
    expect(root.querySelector(".notch-hidden .notch-system")?.textContent).toContain("CPU 23%");
  });

  it("shows the face alone when the user turned the figures off", async () => {
    await controller.changePreferences({ ...backend.savedPreferences, showSystemStats: false });
    backend.deliverSystemStats(CALM);
    expect(root.querySelector(".notch-system")).toBeNull();
  });
});
