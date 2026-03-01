/**
 * GUI debug helper — screenshots both pages for visual inspection.
 *
 * Prerequisites: `make dev` (or `bun run tauri dev`) must be running.
 *
 * Usage:
 *   bunx playwright test e2e/gui-debug.spec.ts
 *
 * Screenshots are saved to ./e2e/screenshots/
 */
import { test, expect } from "@playwright/test";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const screenshotDir = path.resolve(__dirname, "screenshots");

test.describe("GUI Debug — Settings Page", () => {
  test("capture full settings page", async ({ page }) => {
    await page.goto("/");
    // Wait for the settings to load (loading state disappears)
    await page.waitForSelector("h1");
    await page.screenshot({
      path: path.join(screenshotDir, "settings-full.png"),
      fullPage: true,
    });
  });

  test("capture each section", async ({ page }) => {
    await page.goto("/");
    await page.waitForSelector("h1");

    const sections = page.locator(".section");
    const count = await sections.count();

    for (let i = 0; i < count; i++) {
      const section = sections.nth(i);
      const header = await section.locator(".section-header").first().textContent();
      const safeName = (header || `section-${i}`)
        .trim()
        .replace(/[^a-zA-Z0-9]/g, "-")
        .toLowerCase();
      await section.screenshot({
        path: path.join(screenshotDir, `section-${safeName}.png`),
      });
    }
  });
});

test.describe("GUI Debug — Float Window", () => {
  test("capture float overlay", async ({ page }) => {
    await page.goto("/float.html");
    // Float window renders quickly — just wait for root to have content
    await page.waitForSelector("#root");
    // Give it a moment to render
    await page.waitForTimeout(500);
    await page.screenshot({
      path: path.join(screenshotDir, "float-window.png"),
      fullPage: true,
    });
  });
});
