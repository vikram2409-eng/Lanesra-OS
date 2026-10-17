import { test, expect } from '@playwright/test';

/**
 * Basic smoke test: verifies the Team Workspace server is up and serving.
 * WP-0006, REQ-PRE-004.
 * Covers: REQ-PRE-004 (WP-0006 AC-1, AC-3)
 *
 * This is the first E2E test. It proves the harness works:
 * 1. The server starts and responds to health checks
 * 2. The app page loads (HTTP 200)
 *
 * No network access beyond localhost; no API keys. Untested locally — see CI.
 */
test('server serves the app', async ({ page }) => {
  // Navigate to the root. The server should serve the built frontend.
  const response = await page.goto('/');

  // The page should load with HTTP 200 (not a connection error).
  expect(response?.status()).toBe(200);
});
