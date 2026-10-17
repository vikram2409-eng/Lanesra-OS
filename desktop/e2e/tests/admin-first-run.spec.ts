import { test, expect } from '@playwright/test';

/**
 * First-run setup → sign-in → Settings → Feature flags as Administrator.
 * WP-0006, REQ-PRE-004.
 * Covers: REQ-PRE-004 (WP-0006 AC-1)
 *
 * This is the first E2E smoke test. It drives the real Team Workspace server UI:
 * 1. Completes first-run setup (creates the admin account)
 * 2. Signs in as Administrator
 * 3. Navigates to Settings → Feature flags
 * 4. Verifies the feature flags page loads
 *
 * No network access beyond localhost; no API keys (uses the scripted provider
 * via WP-0005 for any AI calls). Untested locally — see CI.
 */
test('admin completes first-run setup and opens feature flags', async ({
  page,
}) => {
  // 1. First-run setup: the server starts with no admin; the setup page creates one.
  await page.goto('/setup');

  // The setup form asks for admin name, email, and password.
  // Selectors are data-testid attributes (stable across UI changes).
  await page.getByTestId('setup-admin-name').fill('E2E Admin');
  await page.getByTestId('setup-admin-email').fill('admin@e2e.test');
  await page.getByTestId('setup-admin-password').fill('e2e-test-password-123');
  await page.getByTestId('setup-submit').click();

  // 2. Should redirect to sign-in; sign in as the new admin.
  await expect(page).toHaveURL(/.*\/signin/);
  await page.getByTestId('signin-email').fill('admin@e2e.test');
  await page.getByTestId('signin-password').fill('e2e-test-password-123');
  await page.getByTestId('signin-submit').click();

  // 3. Should land on the main app; navigate to Settings → Feature flags.
  await expect(page).toHaveURL(/.*\/(app|dashboard)/);
  await page.getByTestId('nav-settings').click();
  await page.getByTestId('settings-feature-flags').click();

  // 4. Feature flags page loads with the flag list.
  await expect(page).toHaveURL(/.*\/settings\/feature-flags/);
  await expect(page.getByTestId('feature-flags-list')).toBeVisible();
});
