import { getCurrentWindow } from "@tauri-apps/api/window";

/** The buttons of a window drawn without the system title bar. */
export interface WindowControls {
  minimize(): Promise<void>;
  toggleMaximize(): Promise<void>;
  close(): Promise<void>;
}

/** Controls the window this page lives in. */
export class TauriWindowControls implements WindowControls {
  async minimize(): Promise<void> {
    await getCurrentWindow().minimize();
  }

  async toggleMaximize(): Promise<void> {
    await getCurrentWindow().toggleMaximize();
  }

  async close(): Promise<void> {
    await getCurrentWindow().close();
  }
}
