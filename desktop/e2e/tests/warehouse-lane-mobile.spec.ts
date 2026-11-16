import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The warehouse demo lane runner at phone widths (mobile CSS fix, 2026-11-15).
 *
 * A missing `}` in styles.css nested ~194 lines of CSS inside
 * `@media(min-width:641px){`, so below 641px the whole lane runner was
 * unstyled: stage numbers jammed on names ("1Pick"), step names ran into
 * their kind ("linesform"), kv rows lost their gaps ("StatusPICKING"), the
 * PO-lines table overflowed its card, and the fixed bottom nav covered
 * content. This spec pins the fix at 390px (and 360px) on the seeded
 * Pick & Pack run on SO-2026-001.
 *
 * Same setup as warehouse-lane.spec.ts: the demo is a static Netlify site,
 * so this spec starts its own zero-dependency static server
 * (desktop/e2e/static-site-server.mjs) on port 4177 and drives
 * /demo/warehouse through it. Each test gets a fresh browser context, so
 * localStorage (the demo store) starts from the seed.
 */

const STATIC_PORT = 4177;
const BASE = `http://localhost:${STATIC_PORT}`;
let server: ChildProcess | null = null;

test.beforeAll(async () => {
  const candidates = [
    path.resolve('e2e/static-site-server.mjs'),
    path.resolve('desktop/e2e/static-site-server.mjs'),
  ];
  const serverPath = candidates.find((p) => fs.existsSync(p));
  if (!serverPath) throw new Error('static-site-server.mjs not found (run from desktop/ or the repo root)');
  server = spawn(process.execPath, [serverPath, String(STATIC_PORT)], { stdio: 'pipe' });
  const deadline = Date.now() + 15000;
  for (;;) {
    try {
      const res = await fetch(`${BASE}/demo/warehouse`);
      if (res.ok) break;
    } catch {
      // not up yet
    }
    if (Date.now() > deadline) throw new Error('static site server did not start');
    await new Promise((r) => setTimeout(r, 250));
  }
});

test.afterAll(async () => {
  server?.kill();
});

async function openSeededPickPackRun(page) {
  await page.goto(`${BASE}/demo/warehouse`);
  // At phone widths the sidebar is hidden behind the hamburger.
  await page.click('#mobileNavToggle');
  await page.click('#sideNav [data-nav="sales_order"]');
  await page.click('[data-open-record="sales_order:so-2026-001"]');
  await page.click('[data-lane-open-run]');
  await expect(page.locator('.view-head h2')).toContainText('SO-2026-001 · Pick & Pack · stage 1 of 5');
  await expect(page.locator('.lane-step-card h3')).toHaveText('Pick the remaining lines');
}

async function noHorizontalOverflow(page, width: number) {
  const overflow = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    innerWidth: window.innerWidth,
  }));
  expect(overflow.scrollWidth, `page overflows at ${width}px`).toBeLessThanOrEqual(overflow.innerWidth);
}

test('390px: the Pick & Pack runner fits, with gaps, and the step button is reachable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openSeededPickPackRun(page);

  await noHorizontalOverflow(page, 390);

  // Stage bar: the flex gap is live (was 0 while the rules were dead).
  const stageGap = await page.evaluate(() => {
    const el = document.querySelector('.lane-stage') as HTMLElement;
    return { display: getComputedStyle(el).display, gap: parseFloat(getComputedStyle(el).gap) };
  });
  expect(stageGap.display).toBe('flex');
  expect(stageGap.gap).toBeGreaterThan(0);

  // Step chips: name and kind are separated (was "linesform").
  const stepGap = await page.evaluate(() => {
    const el = document.querySelector('.lane-step') as HTMLElement;
    return { display: getComputedStyle(el).display, gap: parseFloat(getComputedStyle(el).columnGap) };
  });
  expect(stepGap.display).toBe('flex');
  expect(stepGap.gap).toBeGreaterThan(0);

  // Record kv rows: label and value are separated (was "StatusPICKING").
  const kvGap = await page.evaluate(() => {
    const el = document.querySelector('.lane-side .kv') as HTMLElement;
    return parseFloat(getComputedStyle(el).gap);
  });
  expect(kvGap).toBeGreaterThan(0);

  // PO-lines table stays inside its card (was 504px wide at a 390px viewport).
  const lines = await page.locator('.lane-po-lines-wrap').boundingBox();
  expect(lines).not.toBeNull();
  expect(lines!.x + lines!.width).toBeLessThanOrEqual(390);

  // Complete-step button is reachable: scrolled to the middle of the viewport
  // it must not be covered by the fixed bottom nav.
  const btn = page.locator('#laneCompleteBtn');
  await expect(btn).toBeVisible();
  await btn.evaluate((el) => el.scrollIntoView({ block: 'center' }));
  const covered = await btn.evaluate((el) => {
    const r = el.getBoundingClientRect();
    const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
    return hit !== el && !el.contains(hit);
  });
  expect(covered, 'Complete-step button is covered (bottom nav?)').toBe(false);
});

test('360px: the runner still fits with no horizontal overflow', async ({ page }) => {
  await page.setViewportSize({ width: 360, height: 800 });
  await openSeededPickPackRun(page);
  await noHorizontalOverflow(page, 360);
});
