/**
 * The lane board, "my steps", timing and ageing in the warehouse demo
 * (WP-0713 AC-6, REQ-LANE-019).
 * Covers: REQ-LANE-019 (WP-0713 AC-6).
 *
 * The warehouse demo is a static Netlify site, not the Team Workspace
 * server the Playwright config serves — so this spec starts the shared
 * zero-dependency static server (desktop/e2e/static-site-server.mjs) on
 * port 4181 and drives /demo/warehouse through it. Each test gets a fresh
 * browser context, so localStorage (the demo store) starts from the seed.
 * Ports in use by the other demo specs: 4173 (warehouse-lane,
 * assistant-draft-lane), 4174 (warehouse-lane-blueprint), 4175
 * (lane-publish-dryrun), 4176 (lane-wh-child), 4177 (lane-builder-demo,
 * warehouse-lane-mobile), 4178 (lane-wh-count), 4179 (foundry-demo),
 * 4180 (lane-anytime-actions).
 * Run locally 2026-11-19: 2/2 pass.
 */
import { spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { expect, test } from "@playwright/test";

const STATIC_PORT = 4181;
const BASE = `http://localhost:${STATIC_PORT}`;
let server: ChildProcess | null = null;

test.beforeAll(async () => {
  const candidates = [
    path.resolve("e2e/static-site-server.mjs"),
    path.resolve("desktop/e2e/static-site-server.mjs"),
  ];
  const serverPath = candidates.find((p) => fs.existsSync(p));
  if (!serverPath)
    throw new Error("static-site-server.mjs not found (run from desktop/ or the repo root)");
  server = spawn(process.execPath, [serverPath, String(STATIC_PORT)], { stdio: "pipe" });
  const deadline = Date.now() + 15000;
  for (;;) {
    try {
      const res = await fetch(`${BASE}/demo/warehouse`);
      if (res.ok) break;
    } catch {
      // not up yet
    }
    if (Date.now() > deadline) throw new Error("static site server did not start");
    await new Promise((r) => setTimeout(r, 250));
  }
});

test.afterAll(async () => {
  server?.kill();
});

async function openBoard(page, laneId: string) {
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-board="${laneId}"]`);
  await expect(page.locator(".page-heading h2")).toContainText("Board");
}

test("count lane board shows the active run by stage, my steps and timing", async ({
  page,
}) => {
  // Start the Count lane on the seeded in-progress count; the run waits
  // on the first form step.
  await page.goto(`${BASE}/demo/warehouse`);
  await page.click(`[data-lane-start="lane-wh-count:cc-2026-001"]`);
  await expect(page.locator(".lane-step-card h3")).toBeVisible();

  await openBoard(page, "lane-wh-count");
  await expect(page.locator(".page-heading h2")).toContainText("Count — Board");

  // My steps: the waiting form step, oldest first.
  const mySteps = page.locator("section", { hasText: "My steps" }).first();
  await expect(mySteps).toContainText("Count the lines");
  await expect(mySteps).toContainText("CC-2026-001");

  // Board: the run sits in its first stage, with time in stage.
  const board = page.locator("section", { hasText: "Board — Count" }).first();
  await expect(board).toContainText("Count (1)");
  await expect(board).toContainText("CC-2026-001");
  await expect(board).toContainText("Count the lines");
  await expect(board).toContainText("In stage");

  // Ageing: the same run, longest in stage first.
  const ageing = page.locator("section", { hasText: "Ageing" }).first();
  await expect(ageing).toContainText("CC-2026-001");

  // Timing: the lane has runs and a completion rate.
  const timing = page.locator("section", { hasText: "Timing — Count" }).first();
  await expect(timing).toContainText("Total runs");
  await expect(timing).toContainText("Completion rate");
  await expect(timing).toContainText("Count");
});

test("receiving board shows the seeded run waiting at manager review", async ({
  page,
}) => {
  // PO-2026-005 ships with a run waiting at manager review (approval),
  // so the board and my steps show it without starting anything.
  await openBoard(page, "lane-wh-receiving");
  await expect(page.locator(".page-heading h2")).toContainText("Receive Shipment — Board");

  const mySteps = page.locator("section", { hasText: "My steps" }).first();
  await expect(mySteps).toContainText("Review discrepancy");
  await expect(mySteps).toContainText("PO-2026-005");
  await expect(mySteps).toContainText("Approval");

  const board = page.locator("section", { hasText: "Board — Receive Shipment" }).first();
  await expect(board).toContainText("Hold & review (1)");
  await expect(board).toContainText("PO-2026-005");
  await expect(board).toContainText("Review discrepancy");
});
