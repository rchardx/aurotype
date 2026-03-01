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

  test("capture float recording state", async ({ page }) => {
    await page.goto("/float.html");
    await page.waitForSelector("#root");
    await page.waitForTimeout(300);

    // Inject recording state directly (no Tauri backend needed)
    await page.evaluate(() => {
      const wrapper = document.querySelector('.float-wrapper');
      if (wrapper) {
        // Remove idle class, add recording
        wrapper.className = 'float-wrapper recording';
      }
      // Create the waveform bars if not present
      const iconCenter = document.querySelector('.icon-center');
      if (iconCenter && !iconCenter.querySelector('.waveform-bars')) {
        iconCenter.innerHTML = `
          <div class="waveform-bars">
            <div class="wave-bar" style="height: 16px; animation-delay: 0s"></div>
            <div class="wave-bar" style="height: 28px; animation-delay: 0.1s"></div>
            <div class="wave-bar" style="height: 40px; animation-delay: 0.2s"></div>
            <div class="wave-bar" style="height: 28px; animation-delay: 0.3s"></div>
            <div class="wave-bar" style="height: 16px; animation-delay: 0.4s"></div>
          </div>
        `;
      }
      // Show info panel with recording text
      const infoPanel = document.querySelector('.info-panel');
      if (infoPanel) {
        infoPanel.innerHTML = `
          <div class="status-label">Recording</div>
          <div class="timer-text">0:03</div>
        `;
      }
      // Show pulse ring
      const bubbleContainer = document.querySelector('.bubble-container');
      if (bubbleContainer && !bubbleContainer.querySelector('.pulse-ring')) {
        const ring = document.createElement('div');
        ring.className = 'pulse-ring';
        bubbleContainer.prepend(ring);
      }
    });

    await page.waitForTimeout(500);
    await page.screenshot({
      path: path.join(screenshotDir, "float-recording.png"),
      fullPage: true,
    });
  });
});
