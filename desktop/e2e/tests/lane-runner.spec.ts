import { test, expect, type APIRequestContext, type Page } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The lane runner, first cut (WP-0707).
 * Covers: REQ-LANE-014 (WP-0707 AC-1, AC-2), REQ-LANE-003 (WP-0707 AC-1).
 *
 * Drives the Team Workspace server (the Playwright config's webServer) and
 * the real desktop UI from the same code - server mode. The fixture mirrors
 * the core runner test (desktop/core/tests/process_lane_runner.rs): a
 * `widget` custom object with a `tracking` text field, a `Set Tracking`
 * Action Type, and a published lane, so the runner walks form -> record ->
 * check steps on the straight path with real writes.
 *
 * Seeding goes through the server's own HTTP API (`/api/invoke/<command>`
 * and the async admin routes, the same surface the frontend's `api` module
 * uses), because a fresh server data dir has no lane and no anchor record.
 * Everything seeded is looked up first and reused, so the spec is safe to
 * re-run against a data dir a previous run already seeded.
 *
 * The check step reads `$record.status` (the anchor record's status at run
 * start), not run data or a value a step just wrote: the engine evaluates
 * a check against the run-start snapshot (trigger input), so a check on
 * run data entered mid-run, or on a field a step just wrote, would read
 * the stale start-time value. The straight path still proves form ->
 * record -> check end to end, and the posted write is verified on the
 * record itself after the run completes.
 *
 * Run locally 2026-11-14, green: 2 passed. The server reads its port,
 * data dir and frontend dir from the environment (main.rs: PORT,
 * LANESRA_DATA_DIR, LANESRA_FRONTEND_DIR) and ignores --port /
 * --data-dir flags, so the config's webServer sets those env vars
 * (the old `-- --data-dir /tmp/e2e-data --port 3000` command started
 * the server on :8080 with ./data while Playwright waited on :3000).
 * You can also start the server yourself first - PORT=3000
 * LANESRA_DATA_DIR=/tmp/e2e-data LANESRA_FRONTEND_DIR=./dist
 * ./target/debug/lanesra-server - and let reuseExistingServer pick
 * it up. A re-run against a data dir seeded by an earlier definition
 * needs a fresh dir (rm -rf /tmp/e2e-data): import refuses a lane
 * that already exists and publish can only publish the stored draft,
 * so a changed definition cannot replace the draft in place.
 */

const ADMIN_USERNAME = 'admin';
const ADMIN_PASSWORD = 'e2e-runner-admin-1';
const DATA_DIR = process.env.LANESRA_DATA_DIR ?? '/tmp/e2e-data';
const OBJECT_LABEL = 'Widget';
const OBJECT_PLURAL = 'Widgets';

const RUNNER_LANE_KEY = 'lane-runner-e2e';
const APPROVAL_LANE_KEY = 'lane-approval-e2e';

/** The straight-path lane: form, optional form, record, check, done. */
function runnerLaneDefinition(objectKey: string) {
  return {
    format_version: '1.0.0',
    lane: {
      id: RUNNER_LANE_KEY,
      name: 'Runner Lane',
      object_key: objectKey,
      version: 1,
      status: 'published',
      description: 'The E2E runner lane: capture details, post tracking, check the record, done.',
      run_data: {
        fields: [
          { key: 'carrier', type: 'string', required: true },
          { key: 'note', type: 'string', required: false },
        ],
      },
      stages: [
        {
          id: 'capture',
          name: 'Capture',
          steps: [
            {
              id: 'enter-details',
              name: 'Enter details',
              kind: 'form',
              role: 'r',
              required: true,
              fields: [
                { key: 'carrier', label: 'Carrier', type: 'string', binds_to: 'run_data.carrier', required: true },
                { key: 'tracking', label: 'Tracking', type: 'string', binds_to: 'record.tracking', required: false },
              ],
            },
            {
              id: 'add-note',
              name: 'Add note',
              kind: 'form',
              role: 'r',
              required: false,
              fields: [
                { key: 'note', label: 'Note', type: 'string', binds_to: 'run_data.note', required: false },
              ],
            },
          ],
        },
        {
          id: 'post',
          name: 'Post',
          steps: [
            {
              id: 'post-tracking',
              name: 'Post tracking',
              kind: 'record',
              role: 'r',
              required: true,
              writes: [{ action_type: 'Set Tracking', inputs: { val: '$run_data.carrier' } }],
            },
            {
              id: 'tracking-check',
              name: 'Tracking check',
              kind: 'check',
              required: true,
              check: {
                scope: 'anchor',
                message: 'The record is not Active.',
                condition: { field: '$record.status', operator: 'equals', value: 'Active' },
              },
              // WP-0714 P-09: every check declares both outcomes. A fail
              // branch back to an earlier step would compile to an
              // unbounded cycle (the graph validator only allows cycles
              // through a Loop node), so the fail branch ends the run
              // instead; a retry loop is WP-0720 territory. An undeclared
              // outcome falls through to the next step, so a failing check
              // routes forward in this cut.
              branches: [
                { on: 'pass', target: { stage: 'confirm', step: 'review-summary' } },
                { on: 'fail', end: { outcome: 'failed' } },
              ],
            },
          ],
        },
        {
          id: 'confirm',
          name: 'Confirm',
          steps: [
            {
              id: 'review-summary',
              name: 'Review summary',
              kind: 'form',
              role: 'r',
              required: true,
              ends_run: { outcome: 'done' },
            },
          ],
        },
      ],
    },
  };
}

/** A lane whose first step is an approval, so the run waits in the inbox. */
function approvalLaneDefinition(objectKey: string) {
  return {
    format_version: '1.0.0',
    lane: {
      id: APPROVAL_LANE_KEY,
      name: 'Approval Lane',
      object_key: objectKey,
      version: 1,
      status: 'published',
      description: 'The E2E approval lane: decide first, then post.',
      stages: [
        {
          id: 'decide',
          name: 'Decide',
          steps: [
            {
              id: 'decide',
              name: 'Decide',
              kind: 'approval',
              role: 'r',
              required: true,
              // Approval steps must declare writes; the decision itself
              // writes nothing, so it declares the no-op write, as the
              // core runner test does.
              writes: [{ action_type: 'noop', inputs: {} }],
              branches: [
                { on: 'approved', target: { stage: 'post', step: 'approved-path' } },
                { on: 'rejected', target: { stage: 'post', step: 'rejected-path' } },
              ],
            },
          ],
        },
        {
          id: 'post',
          name: 'Post',
          steps: [
            {
              id: 'approved-path',
              name: 'Approved path',
              kind: 'form',
              role: 'r',
              required: true,
              ends_run: { outcome: 'done' },
            },
            {
              id: 'rejected-path',
              name: 'Rejected path',
              kind: 'form',
              role: 'r',
              required: true,
              ends_run: { outcome: 'held' },
            },
          ],
        },
      ],
    },
  };
}

async function invoke(request: APIRequestContext, command: string, args: Record<string, unknown> = {}) {
  const res = await request.post(`/api/invoke/${command}`, { data: args });
  const body = await res.json();
  if (!body.ok) {
    throw new Error(`invoke ${command} failed: ${body.error?.message ?? JSON.stringify(body)}`);
  }
  return body.data;
}

async function invokeOptional(request: APIRequestContext, command: string, args: Record<string, unknown> = {}) {
  try {
    return await invoke(request, command, args);
  } catch (error) {
    // Seeding is idempotent by design (a re-run re-imports a lane that
    // already exists), but a real failure must not be silent: the verify
    // below would only report "not published" with no cause.
    // eslint-disable-next-line no-console
    console.log(`[lane-runner seed] ${command} skipped: ${error instanceof Error ? error.message : error}`);
    return undefined;
  }
}

let objectKey = 'widget';
let widgetOneId = '';
let widgetTwoId = '';

test.beforeAll(async ({ request }) => {
  // First run: a fresh data dir has no workspace. The server gates
  // first-run setup behind the one-time token it writes to the data dir.
  const workspace = await invoke(request, 'workspace_status');
  if (!workspace) {
    const tokenPath = path.join(DATA_DIR, 'setup-token');
    const setupToken = fs.existsSync(tokenPath) ? fs.readFileSync(tokenPath, 'utf8').trim() : undefined;
    await invoke(request, 'first_run_setup', {
      setup: {
        business_name: 'Lane Runner Co',
        legal_name: null,
        currency_code: 'USD',
        locale: 'en-US',
        timezone: 'UTC',
        default_tax_rate_bp: 0,
        admin_username: ADMIN_USERNAME,
        admin_display_name: 'Admin',
        admin_password: ADMIN_PASSWORD,
        load_sample_data: false,
      },
      ...(setupToken ? { setup_token: setupToken } : {}),
    });
  }

  // Seeding needs an admin session. First-run setup sets one on a fresh
  // data dir, but a re-run against an already-seeded dir skips setup, so
  // log in explicitly either way (the login response sets the session
  // cookie this request context carries for the invokes below).
  await invoke(request, 'login', {
    credentials: { username: ADMIN_USERNAME, password: ADMIN_PASSWORD },
  });

  // The widget object (key is slugified from the singular label).
  const objects = await invoke(request, 'list_custom_objects', { activeOnly: true });
  const existingObject = (objects as Array<{ key: string; plural_label: string }>).find(
    (o) => o.plural_label === OBJECT_PLURAL,
  );
  if (existingObject) {
    objectKey = existingObject.key;
  } else {
    const created = await invoke(request, 'create_custom_object', {
      input: {
        singular_label: OBJECT_LABEL,
        plural_label: OBJECT_PLURAL,
        icon: '▣',
        prefix: 'WID',
        digits: 6,
      },
    });
    objectKey = created.key as string;
  }

  // The tracking field the record step writes.
  const fields = await invoke(request, 'list_custom_field_definitions', { entityType: objectKey, activeOnly: false });
  if (!(fields as Array<{ label: string }>).some((f) => f.label === 'Tracking')) {
    await invoke(request, 'create_custom_field_definition', {
      input: {
        entity_type: objectKey,
        label: 'Tracking',
        field_type: 'text',
        options: [],
        required: false,
        show_in_list: false,
        sort_order: 0,
        min_value: null,
        max_value: null,
        max_length: null,
        regex_pattern: null,
        is_searchable: false,
        is_filterable: false,
        is_reportable: false,
        default_value: null,
        is_unique: false,
        help_text: null,
        placeholder: null,
        is_hidden_by_default: false,
      },
    });
  }

  // The Set Tracking Action Type the record step calls.
  const actionTypes = await invoke(request, 'list_action_types', { targetObjectType: objectKey, activeOnly: false });
  if (!(actionTypes as Array<{ name: string }>).some((a) => a.name === 'Set Tracking')) {
    await invoke(request, 'create_action_type', {
      input: {
        name: 'Set Tracking',
        description: '',
        target_object_type: objectKey,
        required_capability: 'update',
        parameters: [{ key: 'val', label: 'Value', field_type: 'text', required: true, default_value: null }],
        effect: 'set_fields',
        effect_config_json: JSON.stringify({ field_sources: { tracking: 'val' } }),
      },
    });
  }

  // The noop Action Type the approval lane's decision step declares.
  // WP-0714 P-12 requires every named write to be adopted; the write
  // never executes (the compiler only compiles writes on record steps),
  // so it is adopted as a set_status to Active, as the core runner test
  // does (process_lane_runner.rs create_noop_action).
  if (!(actionTypes as Array<{ name: string }>).some((a) => a.name === 'noop')) {
    await invoke(request, 'create_action_type', {
      input: {
        name: 'noop',
        description: '',
        target_object_type: objectKey,
        required_capability: 'update',
        parameters: [],
        effect: 'set_status',
        effect_config_json: JSON.stringify({ target_status: 'Active' }),
      },
    });
  }

  // Both lanes, published. Import/publish are retried idempotently: a lane
  // a previous run already imported just fails the import, and publishing
  // an already-published lane is a no-op for the verify below. One
  // exception: a previous run that left an UNPUBLISHED draft (a publish
  // rule changed since, e.g. WP-0714's P-09/P-12) cannot be repaired
  // through the API — create_version refuses while a draft exists — so
  // reset the data dir (rm -rf /tmp/e2e-data) and re-run in that case.
  for (const definition of [runnerLaneDefinition(objectKey), approvalLaneDefinition(objectKey)]) {
    await invokeOptional(request, 'import_process_lane', { definitionJson: JSON.stringify(definition) });
    await invokeOptional(request, 'publish_process_lane', { laneKey: definition.lane.id });
  }
  const lanes = await invoke(request, 'list_process_lanes', {});
  for (const key of [RUNNER_LANE_KEY, APPROVAL_LANE_KEY]) {
    const lane = (lanes as Array<{ lane_key: string; versions: Array<{ status: string }> }>).find(
      (l) => l.lane_key === key,
    );
    if (!lane || !lane.versions.some((v) => v.status === 'published')) {
      throw new Error(`lane ${key} is not published after seeding`);
    }
  }

  // The anchor records: Widget one runs the straight path, Widget two the
  // approval lane, so a run left active by an interrupted run never blocks
  // the other test (one active run per lane per record).
  const records = await invoke(request, 'list_custom_records', { objectKey });
  for (const [name, slot] of [
    ['Widget one', 'one'],
    ['Widget two', 'two'],
  ] as const) {
    const existing = (records as Array<{ id: string; primary_name: string }>).find((r) => r.primary_name === name);
    const id = existing
      ? existing.id
      : ((await invoke(request, 'create_custom_record', {
          input: { object_key: objectKey, primary_name: name, status: 'Active', owner_user_id: null, notes: null },
        })) as { id: string }).id;
    if (slot === 'one') widgetOneId = id;
    else widgetTwoId = id;
  }

  // Cancel any run a previous (interrupted) suite run left active on these
  // records, so both tests always start a fresh run from the entry point
  // instead of reopening a stale one mid-way.
  const FINISHED = new Set(['completed', 'completed_partial', 'failed', 'cancelled']);
  for (const recordId of [widgetOneId, widgetTwoId]) {
    const listed = await invoke(request, 'list_lane_runs_for_record', { objectKey, recordId });
    for (const run of (listed as { runs: Array<{ run_id: string; status: string; finished_at: string | null }> }).runs) {
      if (run.finished_at === null && !FINISHED.has(run.status)) {
        const res = await request.post(`/api/admin/process-lane-runs/${encodeURIComponent(run.run_id)}/cancel`, {
          data: { reason: 'E2E seed cleanup: cancel a run left active by an earlier run' },
        });
        const body = await res.json();
        if (!body.ok) throw new Error(`cancel ${run.run_id} failed: ${body.error?.message ?? JSON.stringify(body)}`);
      }
    }
  }
});

async function loginAsAdmin(page: Page) {
  await page.request.post('/api/invoke/login', {
    data: { credentials: { username: ADMIN_USERNAME, password: ADMIN_PASSWORD } },
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: OBJECT_PLURAL })).toBeVisible();
}

async function openWidgetsList(page: Page) {
  await page.getByRole('button', { name: OBJECT_PLURAL }).click();
  await expect(page.getByRole('heading', { name: OBJECT_PLURAL })).toBeVisible();
}

test('AC-1: a user completes the straight path from the list without leaving the runner', async ({ page }) => {
  await loginAsAdmin(page);
  await openWidgetsList(page);

  // Start from the list: the row's Start lane run opens the picker card.
  // Two lanes run on this object, so pick the Runner Lane explicitly
  // (the picker defaults to the first lane, Approval Lane).
  const row = page.locator('tr', { hasText: 'Widget one' });
  await row.getByRole('button', { name: 'Start lane run' }).click();
  const startCard = page.locator('div.card', { hasText: 'Start a lane run on' });
  await expect(startCard).toContainText('Widget one');
  await startCard.getByRole('button', { name: 'Start lane run', exact: true }).click();
  await startCard.locator('#lane-run-picker').selectOption({ label: 'Runner Lane' });
  await startCard.getByRole('button', { name: 'Start Runner Lane' }).click();

  // The runner: run label, every stage in the bar, the current stage's steps.
  await expect(page.getByRole('heading', { name: 'Widget one · Runner Lane · stage 1 of 3' })).toBeVisible();
  const stageBar = page.getByRole('list', { name: 'Stage progress' });
  await expect(stageBar).toContainText('Capture');
  await expect(stageBar).toContainText('Post');
  await expect(stageBar).toContainText('Confirm');
  await expect(stageBar.locator('li[aria-current="step"]')).toContainText('Capture');
  const stepList = page.getByRole('group', { name: 'Steps — Capture' });
  await expect(stepList).toContainText('Enter details');
  await expect(stepList).toContainText('Add note');

  // Back previews the previous step only once a step is done; on the first
  // step there is nothing to go back to.
  await expect(page.getByRole('button', { name: 'Back', exact: true })).toHaveCount(0);

  // 1. Enter details (form): carrier is required, tracking is optional.
  await expect(page.getByRole('heading', { name: 'Enter details' })).toBeVisible();
  await page.getByLabel('Carrier *').fill('ACME');
  await page.getByLabel('Tracking').fill('TRK-1');
  await page.getByRole('button', { name: 'Complete step' }).click();

  // 2. Add note is optional: Back now reviews the finished step, and the
  // step list returns to the current one.
  await expect(page.getByRole('heading', { name: 'Add note' })).toBeVisible();
  await page.getByRole('button', { name: 'Back', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Enter details' })).toBeVisible();
  await expect(page.getByText('Back reviews a finished step without changing the run.')).toBeVisible();
  await page.getByRole('button', { name: 'Back to current step' }).click();
  await expect(page.getByRole('heading', { name: 'Add note' })).toBeVisible();
  await page.getByRole('button', { name: 'Skip step' }).click();
  await page.getByLabel('Reason for skipping').fill('nothing to add');
  await page.getByRole('button', { name: 'Confirm skip' }).click();

  // 3. Skipping Add note resumes the run, and the run posts the record
  // step's write and runs the check on its own — the core runner test
  // proves the same ("the record step posted its write on the way out"):
  // the run lands on Review summary, stage 3 of 3, with Post tracking
  // and Tracking check completed in the audit trail. There is no
  // "Post and complete step" click on this path; the runner offers that
  // button only while a run is waiting at a record step.
  await expect(page.getByRole('heading', { name: 'Widget one · Runner Lane · stage 3 of 3' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Review summary' })).toBeVisible();
  await expect(page.getByRole('group', { name: 'Steps — Confirm' })).toContainText('Review summary');
  await expect(page.getByRole('group', { name: 'Audit trail' })).toContainText('Post tracking — completed');
  await expect(page.getByRole('group', { name: 'Audit trail' })).toContainText('Tracking check — completed');

  // The record step's write is listed on the runner.
  await expect(page.locator('.lane-runner-side')).toContainText('Set Tracking');

  // 4. Review summary ends the run.
  await page.getByRole('button', { name: 'Complete step' }).click();
  await expect(page.getByText('Lane Runner Lane · status completed')).toBeVisible();
  const doneBar = page.getByRole('list', { name: 'Stage progress' });
  await expect(doneBar.locator('li')).toHaveCount(3);
  await expect(doneBar).toContainText('✓');

  // The write landed on the record itself: Tracking is the carrier value,
  // not the TRK-1 entered on the form (the record step overwrote it).
  const values = await invoke(page.request, 'get_custom_field_values', { entityId: widgetOneId });
  expect((values as Record<string, unknown>).tracking).toBe('ACME');
});

test('AC-2: a run reaching an approval shows the inbox state, not approval buttons', async ({ page }) => {
  await loginAsAdmin(page);
  await openWidgetsList(page);

  // Start from the record this time: open Widget two, then Start lane run
  // in the record header. Two lanes run on this object, so pick the
  // Approval Lane explicitly.
  const row = page.locator('tr', { hasText: 'Widget two' });
  await row.getByRole('button', { name: 'Edit' }).click();
  await expect(page.getByRole('button', { name: 'View in Object 360' })).toBeVisible();
  await page.getByRole('button', { name: 'Start lane run' }).click();
  await page.locator('#lane-run-picker').selectOption({ label: 'Approval Lane' });
  await page.getByRole('button', { name: 'Start Approval Lane' }).click();

  // The runner names the run and its stage, shows both stages, and says
  // plainly to continue in the approvals inbox.
  await expect(page.getByRole('heading', { name: 'Widget two · Approval Lane · stage 1 of 2' })).toBeVisible();
  const stageBar = page.getByRole('list', { name: 'Stage progress' });
  await expect(stageBar).toContainText('Decide');
  await expect(stageBar).toContainText('Post');
  await expect(page.getByText('This run is waiting on an approval. Continue in the approvals inbox.')).toBeVisible();
  await expect(page.getByText('Approvals are decided in the inbox; this screen does not decide them in this cut.')).toBeVisible();

  // No approval UI on this screen: no Approve/Reject, no Complete step.
  await expect(page.getByRole('button', { name: 'Approve', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Reject', exact: true })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Complete step' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Post and complete step' })).toHaveCount(0);
});
