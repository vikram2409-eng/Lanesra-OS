import { test, expect } from '@playwright/test';

/**
 * Basic smoke test: verifies the Team Workspace server is up and serving the app.
 * WP-0006, REQ-PRE-004.
 * Covers: REQ-PRE-004 (WP-0006 AC-1, AC-3)
 *
 * This is the first E2E test. It proves the harness works:
 * 1. The server starts and responds
 * 2. The app loads in the browser
 * 3. The page has the expected title/content
 *
 * The full first-run → sign-in → feature flags flow will be added once the
 * FirstRun form selectors are verified against the built DOM.
 *
 * No network access beyond localhost; no API keys. Untested locally — see CI.
 */
test('server serves the app', async ({ page }) => {
  // Navigate to the root. The server should serve the built frontend.
  await page.goto('/');

  // The page should load (not a connection error).
  // The app shows either the FirstRun form, the Login form, or the main UI.
  await expect(page).toHaveTitle(/Lanesra/i, { timeout: 30000 });

  // Take a screenshot for debugging (uploaded on failure via trace).
  await page.screenshot();
});
