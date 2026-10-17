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
  await page.goto('/');

  // The FirstRun form (desktop/src/features/firstRun/FirstRun.tsx) uses
  // .form-field divs with a label and input, but labels are not programmatically
  // associated (no for/id). Use structural selectors.
  const businessNameInput = page.locator(
    '.form-field:has-text("Business name") input'
  );
  const usernameInput = page.locator(
    '.form-field:has-text("Administrator username") input'
  );
  const displayNameInput = page.locator(
    '.form-field:has-text("Administrator display name") input'
  );
  const passwordInput = page.locator(
    '.form-field:has-text("Administrator password") input'
  );

  await businessNameInput.fill('E2E Test Business');
  await usernameInput.fill('admin');
  await displayNameInput.fill('E2E Admin');
  await passwordInput.fill('e2e-test-password-123');

  // Submit the form.
  await page.getByRole('button', { name: /set up|create|continue/i }).click();

  // 2. After setup, the app should show the main UI (auto-signed in as admin).
  await expect(page.getByText('E2E Test Business').first()).toBeVisible({
    timeout: 30000,
  });

  // 3. Navigate to Settings → Feature flags.
  await page.getByRole('button', { name: /settings/i }).click();
  await page.getByText('Feature flags').click();

  // 4. Feature flags page loads.
  await expect(page.getByText(/feature flag/i).first()).toBeVisible();
});
