import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E test configuration (WP-0006, REQ-PRE-004).
 * Drives the Team Workspace server's real UI.
 * Covers: REQ-PRE-004 (WP-0006 AC-1, AC-3)
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: 1,
  reporter: [['html', { open: 'never' }], ['list']],
  use: {
    baseURL: process.env.E2E_BASE_URL || 'http://localhost:3000',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  // The webServer starts lanesra-server on a temp data dir for the test run.
  // In CI this is handled by the workflow; locally run:
  //   E2E_BASE_URL=http://localhost:3000 npx playwright test
  webServer: process.env.CI
    ? undefined
    : {
        command: 'cargo run -p lanesra-server -- --data-dir /tmp/e2e-data --port 3000',
        url: 'http://localhost:3000/api/health',
        reuseExistingServer: true,
        timeout: 120000,
      },
});
