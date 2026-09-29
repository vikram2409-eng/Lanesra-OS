#!/usr/bin/env node
// Records a ~2 minute walkthrough of 9 feature areas of the Lanesra OS
// browser demo (/demo), for the homepage's "See it in action" video.
//
// Usage:
//   1. Serve the repo root: `python3 -m http.server 8080` (as README.md
//      documents), from the repo root, in the background.
//   2. Run this script with a Playwright-capable Node:
//        node scripts/record-demo-video.mjs
//   3. The .webm recording lands in OUT_DIR (see below) - Playwright names
//      it itself; this script prints the final path.
//
// Notes on how this script gets into the demo:
//   The public site is client-side-routed by `location.pathname` (see
//   app.js's own `const path=location.pathname...; if(path==='/demo')
//   appShell();`), and in production Netlify's catch-all
//   (`/* -> /index.html status=200`, see netlify.toml) serves index.html's
//   bytes for any path so the client router sees the real URL. A plain
//   `python3 -m http.server` has no such rewrite - requesting /demo
//   directly 404s - so this script installs an equivalent Playwright
//   `route()` handler that fulfills any extensionless document request
//   with the local index.html file, exactly mirroring Netlify's own rule.
//   Every other request (app.js, styles.css, images) still goes to the
//   real local server untouched. Getting into the demo itself is then a
//   real click on the homepage's own "Try Live Demo ->" link
//   (`<a href="/demo">`), not a scripted shortcut.

import { chromium } from '/opt/node22/lib/node_modules/playwright/index.mjs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import fs from 'node:fs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '..');
const BASE_URL = 'http://localhost:8080';
const OUT_DIR = '/tmp/claude-0/demo-video-test/';
fs.mkdirSync(OUT_DIR, { recursive: true });

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// ---- caption overlay -------------------------------------------------------
async function showCaption(page, title) {
  await page.evaluate((t) => {
    const prev = document.getElementById('__demoCaption');
    if (prev) prev.remove();
    let accent = '';
    try {
      accent = getComputedStyle(document.documentElement).getPropertyValue('--brand').trim();
    } catch (e) {}
    if (!accent) accent = '#4f46e5';
    const el = document.createElement('div');
    el.id = '__demoCaption';
    el.textContent = t;
    Object.assign(el.style, {
      position: 'fixed',
      left: '24px',
      bottom: '24px',
      zIndex: 2147483647,
      background: 'rgba(10,10,10,0.82)',
      color: '#ffffff',
      fontSize: '17px',
      fontFamily:
        '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif',
      fontWeight: '600',
      padding: '10px 16px',
      borderRadius: '8px',
      borderLeft: `5px solid ${accent}`,
      boxShadow: '0 8px 24px rgba(0,0,0,0.35)',
      maxWidth: '70vw',
      pointerEvents: 'none',
    });
    document.body.appendChild(el);
  }, title);
}
async function clearCaption(page) {
  await page.evaluate(() => document.getElementById('__demoCaption')?.remove());
}

// Runs one beat: shows the caption, runs the interaction, then pads the
// remaining time so every beat reads on screen for roughly `targetMs`.
async function beat(page, title, targetMs, fn) {
  const start = Date.now();
  console.log(`\n=== BEAT: ${title} ===`);
  await showCaption(page, title);
  await fn();
  const elapsed = Date.now() - start;
  const remaining = targetMs - elapsed;
  if (remaining > 0) await sleep(remaining);
  console.log(`(beat took ${Date.now() - start}ms)`);
}

// ---- real HTML5 drag-and-drop, dispatched at the DOM level ----------------
// Chromium headless does not reliably translate Playwright's synthetic mouse
// moves into native HTML5 dragstart/dragover/drop events. Rather than fake
// the outcome, this dispatches real DragEvents (with a real DataTransfer)
// straight at the exact elements app.js already wires with
// ondragstart/ondragover/ondrop (see wireLayoutDragDrop in app.js) - so the
// app's own moveField() closure runs, unchanged, and really mutates
// data.uiLayouts. This is the same interaction, just driven at the event
// layer instead of the OS mouse layer.
async function dragFieldChipIntoSection(page, fieldLabel, targetSectionIdx) {
  return page.evaluate(
    ({ label, idx }) => {
      const chips = Array.from(document.querySelectorAll('.layout-field-chip[data-section-idx="-1"]'));
      const chip = chips.find((c) => c.textContent.includes(label));
      if (!chip) throw new Error('Field chip not found in Available fields: ' + label);
      const target = document.querySelector(`.layout-field-list[data-section-idx="${idx}"]`);
      if (!target) throw new Error('Target section field list not found: ' + idx);
      const dt = new DataTransfer();
      const fire = (el, type) =>
        el.dispatchEvent(new DragEvent(type, { bubbles: true, cancelable: true, dataTransfer: dt }));
      fire(chip, 'dragstart');
      fire(target, 'dragover');
      fire(target, 'drop');
      fire(chip, 'dragend');
      return true;
    },
    { label: fieldLabel, idx: targetSectionIdx }
  );
}

async function gotoAdminTab(page, tabKey) {
  await page.click('[data-nav="admin"]');
  await page.waitForSelector('[data-admin-open]');
  await page.click(`[data-admin-open="${tabKey}"]`);
  await page.waitForSelector('#adminBody');
  await sleep(300);
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    recordVideo: { dir: OUT_DIR, size: { width: 1280, height: 800 } },
  });

  const downloads = [];
  context.on('page', (p) => {
    p.on('download', (d) => downloads.push(d.suggestedFilename()));
  });

  const page = await context.newPage();
  // No SpeechRecognition constructor -> voiceSpeechSupported() is false ->
  // the demo deterministically shows its own documented "type instead"
  // fallback, exactly like a real machine with no microphone, rather than
  // depending on whatever headless Chromium happens to expose.
  await page.addInitScript(() => {
    delete window.SpeechRecognition;
    delete window.webkitSpeechRecognition;
  });
  page.on('download', (d) => downloads.push(d.suggestedFilename()));
  page.on('console', (msg) => {
    if (msg.type() === 'error') console.log('  [page console error]', msg.text());
  });
  page.on('dialog', async (dialog) => {
    const msg = dialog.message();
    console.log(`  [dialog:${dialog.type()}] ${msg}`);
    if (dialog.type() === 'prompt') {
      if (/Input for/i.test(msg)) {
        await dialog.accept('Tell me about Atlas Construction');
      } else if (/layout name/i.test(msg)) {
        await dialog.accept('Extended layout');
      } else {
        await dialog.accept('Demo');
      }
    } else {
      await dialog.accept();
    }
  });

  // Netlify-style SPA fallback for this local static server - see header
  // comment. Only affects top-level document navigations to an
  // extensionless path; every asset request (app.js, styles.css, images)
  // is left completely untouched.
  await context.route('**/*', async (route) => {
    const req = route.request();
    const url = new URL(req.url());
    const isDoc = req.resourceType() === 'document';
    const looksLikeAsset = /\.[a-z0-9]+$/i.test(url.pathname);
    if (isDoc && url.hostname === 'localhost' && url.pathname !== '/' && !looksLikeAsset) {
      return route.fulfill({
        status: 200,
        contentType: 'text/html; charset=utf-8',
        path: path.join(REPO_ROOT, 'index.html'),
      });
    }
    return route.continue();
  });

  console.log('Navigating to homepage...');
  await page.goto(BASE_URL + '/', { waitUntil: 'load' });
  await sleep(500);

  console.log('Clicking "Try Live Demo ->" (real homepage CTA, href="/demo")...');
  await page.click('a[href="/demo"]');
  await page.waitForSelector('#appSidebar', { timeout: 15000 });
  await sleep(500);

  // =====================================================================
  // BEAT 1 - Low-Code Application Platform
  //   Admin -> Custom fields: add a new field on Company.
  //   Admin -> Screen layouts: drag it from "Available fields" into the
  //   Default layout's Details section, then Publish.
  // =====================================================================
  await beat(page, 'Low-Code Application Platform', 16000, async () => {
    // Visit Screen layouts first so this session's in-memory Default
    // layout for Company materializes from the *current* field set (see
    // ensureLayouts/freshTab in app.js) - a field added after this point
    // will correctly land in "Available fields" below instead of being
    // silently folded into the initial Details section.
    await gotoAdminTab(page, 'layouts'); // "Screen layouts" tab
    await page.waitForSelector('#layoutSections');
    await sleep(900);

    await gotoAdminTab(page, 'fields'); // "Custom fields" tab, defaults to Company
    await page.click('#addField');
    await page.waitForSelector('#cfForm');
    await page.fill('#cfForm [name="label"]', 'Priority Tier');
    await page.selectOption('#cfForm [name="type"]', 'select');
    await page.fill('#cfForm [name="options"]', 'Standard|Priority|VIP');
    await sleep(400);
    await page.click('#cfForm button.btn-primary');
    await page.waitForSelector('.toast');
    await sleep(600);

    await gotoAdminTab(page, 'layouts'); // back to Screen layouts
    await page.waitForSelector('#layoutSections');
    await sleep(400);
    await dragFieldChipIntoSection(page, 'Priority Tier', 0);
    await sleep(700);
    await page.click('#publishLayout');
    await page.waitForSelector('.toast');
    await sleep(500);
  });

  // =====================================================================
  // BEAT 2 - AI Agent Foundry
  //   ensureAdminData() seeds a starter AI Agent Foundry sample the first
  //   time this workspace ever loads (data.workspace.aiFoundrySeeded) -
  //   4 real agents including "Research Analyst" - so this uses that real
  //   seeded agent rather than creating a duplicate: show its persona/
  //   Actions, its Memory, then its Versions tab (Draft -> Test ->
  //   Published lifecycle - v1 already ships Published).
  // =====================================================================
  await beat(page, 'AI Agent Foundry', 15000, async () => {
    await gotoAdminTab(page, 'aiAgents');
    await page.waitForSelector('.table [data-versions-agent]');
    const agentRow = page.locator('.table tr', { hasText: 'Research Analyst' });

    // Persona / Actions - the seeded agent's own Edit form.
    await agentRow.locator('[data-edit-agent]').click();
    await page.waitForSelector('#agentForm');
    await sleep(1200);
    await page.click('#agentForm [data-close]');
    await sleep(300);

    // Memory
    await agentRow.locator('[data-memory-agent]').click();
    await page.waitForSelector('#agentMemoryInput');
    await sleep(500);
    await page.fill(
      '#agentMemoryInput',
      'Atlas Construction and BrightPath Logistics are priority accounts - lead with their open opportunities.'
    );
    await page.click('#saveAgentMemory');
    await page.waitForSelector('.toast');
    await sleep(600);

    // Versions: Draft -> Test -> Published lifecycle
    await agentRow.locator('[data-versions-agent]').click();
    await page.waitForSelector('#addAgentVersionDraft');
    await sleep(700);
    await page.click('#addAgentVersionDraft');
    await page.waitForSelector('#agentVersionForm');
    await sleep(300);
    await page.click('#agentVersionForm button.btn-primary'); // "Create draft" -> v2, status=draft
    await page.waitForSelector('[data-transition-version][data-new-status="test"]');
    await sleep(700);
    await page.click('[data-transition-version][data-new-status="test"]'); // Draft -> Test
    await page.waitForSelector('[data-transition-version][data-new-status="published"]');
    await sleep(700);
    await page.click('[data-transition-version][data-new-status="published"]'); // Test -> Published (deprecates v1)
    await sleep(700);
  });

  // =====================================================================
  // BEAT 3 - Orchestration Pipeline
  //   Same seeding pass gives this workspace 3 real pipelines out of the
  //   box, one per topology - fire the seeded Sequential one ("Company
  //   Briefing": Research Analyst -> Draft Writer) and let its run
  //   result / step history render.
  // =====================================================================
  await beat(page, 'Orchestration Pipeline', 13000, async () => {
    await gotoAdminTab(page, 'aiAgentPipelines');
    await page.waitForSelector('[data-run-pipeline]');
    const pipelineRow = page.locator('.table tr', { hasText: 'Company Briefing' });
    await pipelineRow.scrollIntoViewIfNeeded();
    await sleep(600);

    await pipelineRow.locator('[data-run-pipeline]').click(); // triggers prompt(), auto-accepted by the dialog handler above
    await page.waitForSelector('#pipelineRunWrap .panel');
    await sleep(1800);
  });

  // =====================================================================
  // BEAT 4 - Voice-First Mode
  //   Mic -> set a PIN -> unlock -> full conversation overlay -> "type
  //   instead" (headless Chromium has no real mic) -> a real command
  //   against real seeded data -> the Before/After Confirm step.
  // =====================================================================
  await beat(page, 'Voice-First Mode', 17000, async () => {
    await page.click('#voiceButton');
    await page.waitForSelector('#voicePinSetupForm');
    await page.fill('#voicePinSetupForm [name="pin"]', '1234');
    await page.fill('#voicePinSetupForm [name="pin2"]', '1234');
    await sleep(300);
    await page.click('#voicePinSetupForm button[type="submit"]');
    await page.waitForSelector('#voiceUnlockForm');
    await sleep(400);
    await page.fill('#voiceUnlockForm [name="pin"]', '1234');
    await page.click('#voiceUnlockForm button[type="submit"]');
    await page.waitForSelector('#voiceConversation:not([hidden])');
    await sleep(600);

    await page.click('#voiceConvoTypeToggle'); // "Type instead"
    await page.waitForSelector('#voiceConvoTextRow:not([hidden])');
    // "Cloud Migration" matches both an Opportunity and a Product in the
    // seed data, so this also shows the disambiguation ("which did you
    // mean?") step for free before the Before -> After confirm card.
    await page.fill('#voiceConvoTextInput', 'mark Cloud Migration as Won');
    await sleep(400);
    await page.click('#voiceConvoSendBtn');
    await page.waitForSelector('#voiceConversation [data-voice-candidate="0"]', { timeout: 5000 });
    await sleep(1000);
    await page.click('#voiceConversation [data-voice-candidate="0"]'); // "Cloud Migration (Opportunity · Maya Chen)"
    await page.waitForSelector('#voiceConversation #voiceConfirmBtn', { timeout: 5000 });
    await sleep(1800); // hold on the Before -> After confirm card (Negotiation -> Won)

    await page.click('#voiceConversation #voiceConfirmBtn');
    await sleep(1200);
    await page.click('#voiceConvoClose');
    await sleep(400);
  });

  // =====================================================================
  // BEAT 5 - Access Control v1: Access Inspector
  // =====================================================================
  await beat(page, 'Access Control - Access Inspector', 12000, async () => {
    await gotoAdminTab(page, 'accessInspector');
    await page.waitForSelector('#inspectForm');
    const users = await page.locator('#inspectForm [name="userId"] option').allTextContents();
    console.log('  Users available:', users);
    await page.selectOption('#inspectForm [name="objectKey"]', 'companies');
    await page.selectOption('#inspectForm [name="capability"]', 'canUpdate');
    await sleep(300);
    // pick a real record once the record list has populated for "companies"
    await page.waitForFunction(() => {
      const sel = document.querySelector('#inspectForm [name="recordId"]');
      return sel && sel.options.length > 1;
    });
    await page.selectOption('#inspectForm [name="recordId"]', { index: 1 });
    await sleep(400);
    await page.click('#inspectForm button[type="submit"]');
    await page.waitForSelector('#inspectResult .badge');
    await sleep(1500);
  });

  // =====================================================================
  // BEAT 6 - Integration Hub
  //   No Connections/Connectors/Endpoints are seeded, so add one External
  //   Connection and trigger its "Test request" - a real, simulated
  //   client-side call whose response renders in a modal.
  // =====================================================================
  await beat(page, 'Integration Hub', 13000, async () => {
    await gotoAdminTab(page, 'integrations');
    await page.click('[data-integrations-tab="external"]');
    await page.waitForSelector('#addConnection');
    await page.click('#addConnection');
    await page.waitForSelector('#connectionForm');
    await page.fill('#connectionForm [name="name"]', 'Slack Notifications');
    await page.fill('#connectionForm [name="baseUrl"]', 'https://slack.com/api/chat.postMessage');
    await page.selectOption('#connectionForm [name="method"]', 'POST');
    await page.selectOption('#connectionForm [name="authType"]', 'bearer');
    await page.fill('#connectionForm [name="authValue"]', 'xoxb-demo-token');
    await sleep(400);
    await page.click('#connectionForm button.btn-primary');
    await page.waitForSelector('[data-test-conn]');
    await sleep(500);
    await page.click('[data-test-conn]');
    await page.waitForSelector('#modal pre');
    await sleep(1800);
    await page.click('#modal [data-close]');
    await sleep(400);
  });

  // =====================================================================
  // BEAT 7 - Policy Engine (lives inside the AI Agents admin tab)
  // =====================================================================
  await beat(page, 'Policy Engine', 13000, async () => {
    await gotoAdminTab(page, 'aiAgents');
    await page.waitForSelector('#policyThreshold');
    await page.locator('#policyEngineWrap').scrollIntoViewIfNeeded();
    await sleep(400);
    await page.selectOption('#policyThreshold', 'external_action');
    await sleep(400);
    await page.click('#savePolicy');
    await page.waitForSelector('.toast');
    await sleep(700);

    await page.fill('#overrideToolName', 'send_email');
    await page.selectOption('#overrideRiskLevel', 'external_action');
    await sleep(300);
    await page.click('#setOverride');
    await sleep(1200);
  });

  // =====================================================================
  // BEAT 8 - Industry Packages / App Catalog
  //   Import + Install the Field Service reference package; its new
  //   objects/nav entries appear in the sidebar right after.
  // =====================================================================
  await beat(page, 'Industry Packages', 13000, async () => {
    await gotoAdminTab(page, 'packages');
    await page.waitForSelector('[data-import="field_service"]');
    await page.click('[data-import="field_service"]');
    await page.waitForSelector('[data-install="field_service"]');
    await sleep(600);
    await page.click('[data-install="field_service"]');
    await page.waitForSelector('.toast');
    await sleep(800);
    await page.waitForSelector('#appSwitcher'); // sidebar app switcher now shows Field Service
    await sleep(1200);
  });

  // =====================================================================
  // BEAT 9 - Solution Packages & Deployment
  //   Deployment Management -> Solutions: create one, curate a component,
  //   then Export it (a real client-side file download).
  // =====================================================================
  await beat(page, 'Solution Packages & Deployment', 14000, async () => {
    await gotoAdminTab(page, 'solutions');
    await page.click('[data-solutions-tab="solutions"]');
    await page.waitForSelector('#newSolution');
    await page.click('#newSolution');
    await page.waitForSelector('#solutionForm');
    await page.fill('#solutionForm [name="name"]', 'Field Service Starter');
    await page.fill('#solutionForm [name="description"]', 'Everything needed to ship Field Service to another workspace.');
    await sleep(400);
    await page.click('#solutionForm button.btn-primary');
    await page.waitForSelector('[data-add-member]');
    await sleep(500);
    await page.click('[data-add-member]');
    await page.waitForSelector('#modal [data-close]');
    await sleep(500);
    await page.click('#modal [data-close]');
    await sleep(400);

    const [download] = await Promise.all([
      page.waitForEvent('download'),
      page.click('[data-export-solution]'),
    ]);
    console.log('  Downloaded file:', download.suggestedFilename());
    await page.waitForSelector('.toast');
    await sleep(1500);
  });

  await clearCaption(page);
  await sleep(500);

  console.log('\nClosing context to flush video...');
  await context.close();
  await browser.close();

  console.log('\nDownload events observed:', downloads);
  console.log('Done. Video directory:', OUT_DIR);
}

main().catch((err) => {
  console.error('FAILED:', err);
  process.exit(1);
});
