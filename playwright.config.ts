import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 15000,
  use: {
    baseURL: "http://localhost:1420",
    // Larger viewport to see the full Settings page
    viewport: { width: 800, height: 900 },
    // Screenshot on failure
    screenshot: "only-on-failure",
  },
  // Don't start a web server — user runs `make dev` manually
  webServer: undefined,
});
