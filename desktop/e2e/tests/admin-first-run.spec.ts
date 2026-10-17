import { test, expect } from '@playwright/test';

/**
 * First-run setup → sign-in → Settings → Feature flags as Administrator.
 * WP-0006, REQ-PRE-004.
 * Covers: REQ-PRE-004 (WP-0006 AC-1)
 *
 * This is the first E2E smoke test. It drives the real Team Workspace server UI:
 * 1. Completes first-run setup (creates the admin account via the FirstRun form)
 * 2. The app auto-signs in as the new admin after setup
 * 3. Navigates to Settings → Feature flags
 * 4. Verifies the feature flags page loads
 *
 * No network access beyond localhost; no API keys. Untested locally — see CI.
 */
test('admin completes first-run setup and opens feature flags', async ({
  page,
}) => {
  // 1. First-run setup: the app shows the FirstRun form when no workspace exists.
  // The root URL renders the FirstRun component (desktop/src/features/firstRun/FirstRun.tsx).
  await page.goto('/');

  // Fill the first-run form using label selectors (the form has no data-testids).
  await page.getByLabel('Business name').fill('E2E Test Business');
  await page.getByLabel('Administrator username').fill('admin');
  await page.getByLabel('Administrator display name').fill('E2E Admin');
  await page
    .getByLabel('Administrator password (min 8 characters)')
    .fill('e2e-test-password-123');

  // Submit the form.
  await page.getByRole('button', { name: /set up|create|continue/i }).click();

  // 2. After setup, the app should show the main UI (auto-signed in as admin).
  // Wait for the dashboard or app shell to appear.
  await expect(page.getByText('E2E Test Business').first()).toBeVisible({
    timeout: 30000,
  });

  // 3. Navigate to Settings → Feature flags.
  // The Settings is accessed via the AdminPanel (desktop/src/features/settings/Settings.tsx).
  await page.getByRole('button', { name: /settings/i }).click();
  await page.getByText('Feature flags').click();

  // 4. Feature flags page loads.
  await expect(page.getByText(/feature flag/i).first()).toBeVisible();
});
