// Child lanes in the online warehouse demo (WP-0716 AC-5).
//
// The Receiving lane hands the supplier check to the Supplier Check lane
// (step 1.3, Verify the supplier, anchor $record.supplier_id): the parent
// waits in waiting_child, shows the child run, and resumes when the child
// finishes; cancelling asks whether the child runs on or is cancelled too.
//
// Covers: REQ-LANE-023 (WP-0716 AC-5).
//
// Spins up the same static server as the other demo specs, on its own port
// (4176), and exercises the demo through the browser.

import { test, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import type { ChildProcessWithoutNullStreams } from 'node:child_process';

const DEMO_URL = 'http://127.0.0.1:4176/demo/warehouse/';
const PO = 'po-2026-001';
const SUPPLIER = 'sup-01';

let server: ChildProcessWithoutNullStreams;

test.beforeAll(async () => {
  server = spawn('node', ['e2e/static-site-server.mjs', '4176'], { cwd: process.cwd() });
  server.stdout.resume();
  server.stderr.resume();
  await once(server.stdout, 'data');
});

test.afterAll(() => {
  server.kill();
});

test.beforeEach(async ({ page }) => {
  await page.goto(DEMO_URL);
});

async function expectStep(page, name: string) {
  await expect(page.locator('.lane-step-card h3')).toHaveText(name);
}

async function completeStep(page) {
  await expect(page.locator('#laneCompleteBtn')).toBeVisible();
  await page.click('#laneCompleteBtn');
}

async function openPoLane(page) {
  await page.click('#sideNav [data-nav="purchase_order"]');
  await page.click(`[data-open-record="purchase_order:${PO}"]`);
  await page.click(`[data-lane-start="lane-wh-receiving:${PO}"]`);
  await expectStep(page, 'Confirm shipment arrived');
}

async function walkToChildStep(page) {
  await openPoLane(page);
  await page.fill('[data-lane-field="carrier"]', 'FastFreight');
  await page.fill('[data-lane-field="delivery_note_number"]', 'DN-7781');
  await page.fill('[data-lane-field="received_date"]', '2026-11-13');
  await completeStep(page);
  await page.fill('[data-lane-field="pol-001-1:delivery_note_qty"]', '40');
  await page.fill('[data-lane-field="pol-001-2:delivery_note_qty"]', '30');
  await completeStep(page);
  await expectStep(page, 'Verify the supplier');
  await expect(page.locator('.lane-waiting')).toContainText('waiting on Supplier Check — Verify');
}

async function completeChildRun(page) {
  await page.click('.lane-waiting [data-lane-open-run]');
  await expectStep(page, 'Confirm the supplier contact');
  await page.fill('[data-lane-field="contact_name"]', 'Priya Nair');
  await completeStep(page);
  // The check passes on its own; the record step marks the check complete.
  await expectStep(page, 'Mark the supplier check complete');
  await completeStep(page);
}

async function finishParentRun(page) {
  // The parent resumes past the child step at stage 2. The rest of the walk
  // is the AC-1 walk (warehouse-lane.spec.ts).
  await expectStep(page, 'Enter received quantities');
  await page.fill('[data-lane-field="pol-001-1:qty_received"]', '40');
  await page.fill('[data-lane-field="pol-001-1:qty_rejected"]', '0');
  await page.fill('[data-lane-field="pol-001-2:qty_received"]', '30');
  await page.fill('[data-lane-field="pol-001-2:qty_rejected"]', '0');
  await completeStep(page);
  await expectStep(page, 'Stage the goods');
  await expect(page.locator('[data-lane-field="staging_bin"]')).toHaveValue('loc-rec-01');
  await completeStep(page);
  await expectStep(page, 'Post receipt moves');
  await completeStep(page);
  await expectStep(page, 'Confirm put-away');
  await expect(page.locator('[data-lane-field="pol-001-1:putaway_bin"]')).toHaveValue('loc-a-01-03');
  await expect(page.locator('[data-lane-field="pol-001-2:putaway_bin"]')).toHaveValue('loc-a-01-03');
  await completeStep(page);
  await expectStep(page, 'Post put-away moves');
  await completeStep(page);
  await expectStep(page, 'Review summary');
  await completeStep(page);
  await expectStep(page, 'Close the PO');
  await completeStep(page);
}

async function openSupplierRecord(page) {
  await page.click('#sideNav [data-nav="supplier"]');
  await page.click(`[data-open-record="supplier:${SUPPLIER}"]`);
}

test('AC-5: the parent waits on the child and resumes when it finishes', async ({ page }) => {
  await walkToChildStep(page);

  // The parent has no Complete button while it waits.
  await expect(page.locator('#laneCompleteBtn')).toHaveCount(0);

  await completeChildRun(page);
  await finishParentRun(page);

  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');
  await expect(page.locator('.lane-audit')).toContainText('Child Supplier Check finished — completed.');
  await expect(page.locator('.lane-side')).toContainText('RECEIVED');

  // End state matches the worked example: stock 88 and 52, four moves.
  await expect(page.locator('.lane-po-lines')).toContainText('88');
  await expect(page.locator('.lane-po-lines')).toContainText('52');
  const writes = page.locator('.lane-writes');
  await expect(writes).toContainText('MV-2026-016 RECEIPT HG-1001 × 40');
  await expect(writes).toContainText('MV-2026-017 RECEIPT HG-1002 × 30');
  await expect(writes).toContainText('MV-2026-018 PUTAWAY HG-1001 × 40');
  await expect(writes).toContainText('MV-2026-019 PUTAWAY HG-1002 × 30');
});

test('AC-5: cancelling the parent cancels the child too', async ({ page }) => {
  await walkToChildStep(page);

  await page.click('#laneCancelBtn');
  await expect(page.locator('.lane-cancel-prompt')).toContainText('Cancel the child run too');
  await page.click('#laneCancelChildBtn');
  await expect(page.locator('.lane-outcome')).toContainText('Run cancelled — cancelled');

  // The child is cancelled as well: the supplier record offers a fresh
  // start, not an open run.
  await openSupplierRecord(page);
  await expect(page.locator(`[data-lane-start="lane-wh-supplier-check:${SUPPLIER}"]`)).toBeVisible();
});

test('AC-5: cancelling the parent can keep the child running', async ({ page }) => {
  await walkToChildStep(page);

  await page.click('#laneCancelBtn');
  await expect(page.locator('.lane-cancel-prompt')).toContainText('Cancel the child run too');
  await page.click('#laneKeepChildBtn');
  await expect(page.locator('.lane-outcome')).toContainText('Run cancelled — cancelled');

  // The child is still active on the supplier record.
  await openSupplierRecord(page);
  await expect(page.locator('[data-lane-open-run]')).toBeVisible();
  await page.click('[data-lane-open-run]');
  await expectStep(page, 'Confirm the supplier contact');
  await page.fill('[data-lane-field="contact_name"]', 'Priya Nair');
  await completeStep(page);
  await expectStep(page, 'Mark the supplier check complete');
  await completeStep(page);
  await expect(page.locator('.lane-outcome')).toContainText('Run completed — done');

  // The parent stays cancelled: a fresh start is offered, not a resume.
  await page.click('#sideNav [data-nav="purchase_order"]');
  await page.click(`[data-open-record="purchase_order:${PO}"]`);
  await expect(page.locator(`[data-lane-start="lane-wh-receiving:${PO}"]`)).toBeVisible();
});

test('AC-5: cancelling the child resumes the parent', async ({ page }) => {
  await walkToChildStep(page);

  await page.click('.lane-waiting [data-lane-open-run]');
  await expectStep(page, 'Confirm the supplier contact');
  await page.click('#laneCancelBtn');
  await page.click('#laneCancelConfirmBtn');

  // The child is cancelled; the parent resumes past the child step.
  await expectStep(page, 'Enter received quantities');
  await expect(page.locator('.lane-audit')).toContainText('Child Supplier Check finished — cancelled.');
});
