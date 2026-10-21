import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const repositoryRoot = fileURLToPath(new URL("../..", import.meta.url));

// Tauri expects a fixed dev server port and serves the built files from `dist`.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: "0.0.0.0",
    fs: { allow: [repositoryRoot] },
  },
  envPrefix: ["VITE_", "TAURI_ENV_"],
  build: {
    target: "es2023",
    outDir: "dist",
    emptyOutDir: true,
    // Two windows, two pages: the island and the advanced settings.
    rolldownOptions: {
      input: {
        island: fileURLToPath(new URL("index.html", import.meta.url)),
        settings: fileURLToPath(new URL("settings.html", import.meta.url)),
      },
    },
  },
  test: {
    environment: "happy-dom",
    include: ["tests/**/*.test.ts"],
  },
});
