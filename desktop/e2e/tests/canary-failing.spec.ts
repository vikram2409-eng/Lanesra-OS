import { test, expect } from '@playwright/test';

/**
 * DELIBERATELY FAILING canary spec for WP-0006 AC-2.
 * This proves the harness uploads a trace artifact on failure.
 * Like PR #650 (canary migration), this branch is closed unmerged.
 * Covers: REQ-PRE-004 (WP-0006 AC-2)
 */
test('canary: this spec always fails', async ({ page }) => {
  await page.goto('/');
  // Deliberate failure: expect a status that will never match
  expect(999).toBe(200);
});
