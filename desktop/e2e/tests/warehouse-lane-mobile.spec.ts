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
 *
 * Round 2 (2026-11-15): the record pages were still laid out 818px wide
 * inside the 390px .main-area scroll container — the record grid's `1fr`
 * column took the Lines table's 760px min-width as its min-content, so
 * every card ran off the right edge, the topbar search shrank to ~97px
 * ("Searc"), and headings scrolled half under the sticky topbar. The
 * record-page tests below pin the fix on PO-2026-001 and SO-2026-001.
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

async function openRecord(page, nav: string, recordSel: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  // At phone widths the sidebar is hidden behind the hamburger.
  await page.click('#mobileNavToggle');
  await page.click(`#sideNav [data-nav="${nav}"]`);
  await page.click(`[data-open-record="${recordSel}"]`);
  await page.waitForTimeout(400);
}

async function noHorizontalOverflow(page, width: number) {
  const overflow = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    innerWidth: window.innerWidth,
  }));
  expect(overflow.scrollWidth, `page overflows at ${width}px`).toBeLessThanOrEqual(overflow.innerWidth);
}

async function noInnerOverflow(page, width: number) {
  // Phones scroll inside .main-area (position:fixed, overflow-y:auto), not
  // the document — the document never overflows, so measure the container
  // the user actually scrolls. Round 2: the record grid widened it to 818px.
  const overflow = await page.evaluate(() => {
    const ma = document.querySelector('.main-area') as HTMLElement;
    return { scrollWidth: ma.scrollWidth, innerWidth: window.innerWidth };
  });
  expect(overflow.scrollWidth, `.main-area overflows at ${width}px`).toBeLessThanOrEqual(overflow.innerWidth);
}

async function recordPageFits(page, width: number) {
  await noHorizontalOverflow(page, width);
  await noInnerOverflow(page, width);

  // Topbar search is usable: the word "Search" fits (was ~97px wide,
  // showing "Searc") and the low-priority actions are hidden at <=480px.
  const search = await page.locator('#globalSearch').boundingBox();
  expect(search).not.toBeNull();
  expect(search!.width, 'search input too narrow').toBeGreaterThanOrEqual(180);
  await expect(page.locator('#voiceButton')).toBeHidden();
  await expect(page.locator('#helpButton')).toBeHidden();
  await expect(page.locator('#themeToggle')).toBeHidden();

  // Lane card title text stays inside the viewport (was clipped at the
  // right edge when the page was 818px wide). Measure the text itself,
  // not the block box, which spans the whole card head.
  const laneRight = await page.evaluate(() => {
    const h = [...document.querySelectorAll('.card-head h3')].find((el) =>
      el.textContent!.startsWith('Process Lane'));
    if (!h) return -1;
    const range = document.createRange();
    range.selectNodeContents(h);
    const r = range.getBoundingClientRect();
    return r.x + r.width;
  });
  expect(laneRight).toBeGreaterThan(0);
  expect(laneRight, 'lane card title runs off the right edge').toBeLessThanOrEqual(width);

  // Section + record headings are contained.
  for (const sel of ['.card-head h3', '.record-name h2']) {
    const boxes = await page.locator(sel).evaluateAll((els) =>
      els.map((el) => el.getBoundingClientRect().right));
    for (const right of boxes) {
      expect(right, `${sel} runs off the right edge`).toBeLessThanOrEqual(width);
    }
  }

  // Lines table stays inside its card: the wrapper is viewport-wide and
  // the 760px table scrolls inside it (it used to widen the whole page).
  const wrap = await page.locator('.prb2-lines .table-wrap').boundingBox();
  expect(wrap).not.toBeNull();
  expect(wrap!.x + wrap!.width, 'Lines table wrapper overflows').toBeLessThanOrEqual(width);
  const wrapScroll = await page.evaluate(() => {
    const el = document.querySelector('.prb2-lines .table-wrap') as HTMLElement;
    return { scrollWidth: el.scrollWidth, clientWidth: el.clientWidth };
  });
  expect(wrapScroll.scrollWidth, 'Lines table should scroll inside its card').toBeGreaterThan(wrapScroll.clientWidth);

  // A section scrolled to the top lands below the sticky topbar (was
  // half-hidden under it: scroll targets used to ignore the 58px bar
  // because scroll-padding lived on html, which does not scroll here).
  const landed = await page.evaluate(() => {
    const bar = document.querySelector('.topbar')!.getBoundingClientRect();
    const card = document.querySelector('.prb2-lines') as HTMLElement;
    card.scrollIntoView({ block: 'start' });
    return { cardTop: card.getBoundingClientRect().top, barBottom: bar.bottom };
  });
  expect(landed.cardTop, 'scrolled-to card hides under the topbar').toBeGreaterThanOrEqual(landed.barBottom);

  // Last visible content clears the fixed bottom nav at max scroll.
  const clearance = await page.evaluate(() => {
    const ma = document.querySelector('.main-area') as HTMLElement;
    const nav = document.querySelector('.bottom-nav') as HTMLElement;
    ma.scrollTop = ma.scrollHeight;
    const navTop = nav.getBoundingClientRect().top;
    const isVisible = (el: Element): boolean => {
      if (!el.getClientRects().length) return false;
      if (getComputedStyle(el).visibility === 'hidden') return false;
      let n: Element | null = el;
      while (n && n !== document.body) {
        if (getComputedStyle(n).display === 'none') return false;
        n = n.parentElement;
      }
      return true;
    };
    const leaves = [...document.querySelectorAll('#view *')].filter(
      (el) => el.children.length === 0 && (el.textContent || '').trim() && isVisible(el),
    ) as HTMLElement[];
    let worst = { text: '', bottom: -Infinity };
    for (const el of leaves) {
      const b = el.getBoundingClientRect().bottom;
      if (b > worst.bottom) worst = { text: (el.textContent || '').trim().slice(0, 40), bottom: b };
    }
    return { lastText: worst.text, lastBottom: worst.bottom, navTop };
  });
  expect(clearance.lastBottom, `last content "${clearance.lastText}" slides under the bottom nav`).toBeLessThanOrEqual(clearance.navTop);
}

test('390px: the Pick & Pack runner fits, with gaps, and the step button is reachable', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openSeededPickPackRun(page);

  await noHorizontalOverflow(page, 390);

  // Stage swim-lane header: the flex gap is live (num and name separated).
  const stageGap = await page.evaluate(() => {
    const el = document.querySelector('.lane-swim-head') as HTMLElement;
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

test('390px: the PO-2026-001 record page fits — search, headings, Lines table, bottom clearance', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openRecord(page, 'purchase_order', 'purchase_order:po-2026-001');
  await expect(page.locator('.record-name h2')).toContainText('PO-2026-001');
  await recordPageFits(page, 390);
});

test('390px: the SO-2026-001 record page fits — search, headings, Lines table, bottom clearance', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await openRecord(page, 'sales_order', 'sales_order:so-2026-001');
  await expect(page.locator('.record-name h2')).toContainText('SO-2026-001');
  await recordPageFits(page, 390);
});
