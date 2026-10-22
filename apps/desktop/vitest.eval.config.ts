import { defineConfig } from "vitest/config";

// Evals call the real AI providers with the user's key: run them on purpose, never in `make test`.
export default defineConfig({
  test: {
    environment: "node",
    include: ["tests/evals/**/*.eval.ts"],
  },
});
