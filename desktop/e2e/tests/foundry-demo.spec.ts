import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The Agent Foundry demo at /demo/foundry (WP-0653).
 * Covers: REQ-PG-016 (WP-0653 AC-4).
 *
 * The Foundry demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts its own
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4179 and drives /demo/foundry through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Untested locally — see CI.
 */

const STATIC_PORT = 4179;
const BASE = `http://localhost:${STATIC_PORT}`;
let server: ChildProcess | null = null;

const FOUNDRY_ITEMS = [
  { key: 'f_agents', label: 'Agents', band: 'Create' },
  { key: 'f_models', label: 'Models', band: 'Create' },
  { key: 'f_services', label: 'Services', band: 'Create' },
  { key: 'f_tools', label: 'Tools', band: 'Create' },
  { key: 'f_knowledge', label: 'Knowledge', band: 'Create' },
  { key: 'f_memory', label: 'Memory', band: 'Create' },
  { key: 'f_guardrails', label: 'Guardrails', band: 'Create' },
  { key: 'f_orchestration', label: 'Orchestration', band: 'Create' },
  { key: 'f_monitor', label: 'Monitor', band: 'Observe' },
  { key: 'f_traces', label: 'Traces', band: 'Observe' },
  { key: 'f_data', label: 'Data', band: 'Improve' },
  { key: 'f_evaluations', label: 'Evaluations', band: 'Improve' },
  { key: 'f_optimization', label: 'Optimization', band: 'Improve' },
] as const;

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
      const res = await fetch(`${BASE}/demo/foundry`);
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

async function gotoFoundry(page) {
  const errors: string[] = [];
  page.on('pageerror', (err) => errors.push(String(err)));
  await page.goto(`${BASE}/demo/foundry`);
  await expect(page.locator('#sideNav .nav-item').first()).toBeVisible();
  return errors;
}

test('AC-4: the Foundry shell shows Create / Observe / Improve with all thirteen items', async ({ page }) => {
  const errors = await gotoFoundry(page);

  // The brand pill marks this demo as the Foundry, not the sales demo.
  await expect(page.locator('.demo-pill')).toHaveText('FOUNDRY');
  await expect(page.locator('.page-heading h2')).toHaveText('Agent Foundry');

  // Three bands, in order, each with its items.
  const labels = page.locator('#sideNav .side-label');
  await expect(labels).toHaveText(['Create', 'Observe', 'Improve']);
  for (const item of FOUNDRY_ITEMS) {
    const button = page.locator(`#sideNav [data-fnav="${item.key}"]`);
    await expect(button).toBeVisible();
    await expect(button.locator('.nav-label')).toHaveText(item.label);
  }
  await expect(page.locator('#sideNav [data-fnav="f_home"] .nav-label')).toHaveText('Overview');

  // Zero page errors loading the shell and its home view.
  expect(errors, `page errors on /demo/foundry: ${errors.join('; ')}`).toEqual([]);
});

test('AC-4: a reused screen and a native panel both open under the Foundry menu', async ({ page }) => {
  await gotoFoundry(page);

  // Reused screen: Agents renders the real admin Agents tab.
  await page.click('#sideNav [data-fnav="f_agents"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Agents');
  await expect(page.locator('#view')).toContainText('Research Analyst');

  // Native panel: Traces renders the OTLP-style span trees from the store.
  await page.click('#sideNav [data-fnav="f_traces"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Traces');
  await expect(page.locator('#view')).toContainText('trc_7f3a21');
  await expect(page.locator('#view')).toContainText('agent.run');
  await expect(page.locator('#view')).toContainText('guardrail.input.jailbreak');
  // AC-6 depth: each trace names its tool calls, guardrail checks and
  // delegations, and every span carries a duration bar.
  await expect(page.locator('#view')).toContainText('2 tool calls · 1 guardrail check · 0 delegations');

  // Native panel: Monitor shows live-foundry KPIs and recent runs.
  await page.click('#sideNav [data-fnav="f_monitor"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Monitor');
  await expect(page.locator('#view')).toContainText('Active agents');
  await expect(page.locator('#view')).toContainText('Recent runs');
  // AC-6 depth: runs broken down by agent, and the approvals inbox state.
  await expect(page.locator('#view')).toContainText('Runs by agent');
  await expect(page.locator('#view')).toContainText('Research Analyst');
  await expect(page.locator('#view')).toContainText('No approvals waiting');
});

test('AC-6: the Orchestration view lists the real pipelines and agent teams', async ({ page }) => {
  await gotoFoundry(page);

  await page.click('#sideNav [data-fnav="f_orchestration"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Orchestration');
  // One pipeline per topology, with their steps named by agent.
  await expect(page.locator('#view')).toContainText('Company Briefing');
  await expect(page.locator('#view')).toContainText('Consensus Estimate');
  await expect(page.locator('#view')).toContainText('Draft & Review Loop');
  await expect(page.locator('#view')).toContainText('sequential');
  await expect(page.locator('#view')).toContainText('consensus');
  await expect(page.locator('#view')).toContainText('peer_review');
  // The seeded agent team, with its agent nodes resolved to names.
  await expect(page.locator('#view')).toContainText('Sales Ops Team');
  await expect(page.locator('#view')).toContainText('Ops Supervisor');
  await expect(page.locator('#view')).toContainText('Research Analyst');

  // The home Orchestration card links here, not to Traces.
  await page.click('#sideNav [data-fnav="f_home"]');
  await expect(page.locator('#view')).toContainText('Orchestration');
  await page.click('[data-fgo="f_orchestration"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Orchestration');
});

test('AC-6: the theme toggle flips light and dark, and the choice persists', async ({ page }) => {
  await gotoFoundry(page);

  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.click('#themeToggle');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'dark');
  await expect(page.locator('#view')).toContainText('Agent Foundry');
  await page.click('#themeToggle');
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
});

test('AC-6: at phone width there is no horizontal scroll and the mobile menu works', async ({ browser }) => {
  const context = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await context.newPage();
  try {
    await page.goto(`${BASE}/demo/foundry`);
    await expect(page.locator('#sideNav .nav-item').first()).toBeVisible();
    const noHScroll = async () =>
      await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth);
    expect(await noHScroll(), 'home view scrolls horizontally at 390px').toBe(true);

    // At phone width the sidebar is a collapsed icon rail and the lower
    // items sit below the fold - navigation goes through the drawer, which
    // is the designed mobile flow (the rail alone cannot reach them).
    await page.click('#mobileNavToggle');
    await expect(page.locator('#appSidebar')).toHaveClass(/open/);
    await expect(page.locator('#mobileNavToggle')).toHaveAttribute('aria-expanded', 'true');
    await page.click('#sideNav [data-fnav="f_traces"]');
    await expect(page.locator('.page-heading h2')).toHaveText('Traces');
    expect(await noHScroll(), 'Traces view scrolls horizontally at 390px').toBe(true);

    // Picking an item navigates and closes the drawer again.
    await page.click('#mobileNavToggle');
    await page.click('#sideNav [data-fnav="f_monitor"]');
    await expect(page.locator('.page-heading h2')).toHaveText('Monitor');
    await expect(page.locator('#appSidebar')).not.toHaveClass(/open/);
  } finally {
    await context.close();
  }
});

test('AC-4: the Optimization hero climbs from the baseline to the kept candidate', async ({ page }) => {
  await gotoFoundry(page);

  // The hero is on the Overview home view.
  await expect(page.locator('#view')).toContainText('Run details · Improvement');
  await expect(page.locator('#view')).toContainText('Candidate kept');
  await expect(page.locator('#view')).toContainText('0.71');
  await expect(page.locator('#view')).toContainText('0.82');

  // The Optimization item shows the same hero on its own page.
  await page.click('#sideNav [data-fnav="f_optimization"]');
  await expect(page.locator('.page-heading h2')).toHaveText('Optimization');
  await expect(page.locator('#view')).toContainText('Run details · Improvement');
  await expect(page.locator('#view')).toContainText('Task-weighted');
});

test('AC-4: the sales and warehouse demos still load untouched', async ({ page }) => {
  // Sales demo: the default demo, with its own search placeholder.
  await page.goto(`${BASE}/demo`);
  await expect(page.locator('#globalSearch')).toHaveAttribute('placeholder', /companies/);
  await expect(page.locator('.demo-pill')).toHaveText('DEMO');

  // Warehouse demo: its own search placeholder and dashboard.
  await page.goto(`${BASE}/demo/warehouse`);
  await expect(page.locator('#globalSearch')).toHaveAttribute('placeholder', /SKUs/);
  await expect(page.locator('#view')).toContainText('Cycle count in progress');
});
