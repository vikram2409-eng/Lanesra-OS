import { test, expect, type APIRequestContext, type Page } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';

/**
 * Publish checks and the dry run (WP-0714).
 * Covers: REQ-LANE-021 (WP-0714 AC-1, AC-2, AC-4).
 *
 * Drives the Team Workspace server (the Playwright config's webServer)
 * and the real desktop UI from the same code - server mode. The fixture
 * mirrors the lane-runner spec (desktop/e2e/tests/lane-runner.spec.ts):
 * the same `widget` custom object, `tracking` text field and
 * `Set Tracking` Action Type, reused when a previous run already seeded
 * them, so the spec is safe to re-run against a data dir a previous run
 * already seeded. Both lanes stay drafts - publish checks and the dry
 * run are the author's pre-publish tools.
 *
 * Seeding goes through the server's own HTTP API (`/api/invoke/<command>`
 * and the async admin routes, the same surface the frontend's `api` module
 * uses). Everything seeded is looked up first and reused.
 *
 * NOTE: the publish-check and dry-run commands (`check_process_lane_publish`,
 * `dry_run_process_lane`) are the new WP-0714 commands; `publish_lane` refuses
 * with every failed check named (WP-0714 Progress log, 2026-11-14; verified in
 * `core::services::process_lane_service::publish_lane`). The Tauri commands
 * and server dispatch arms have landed and match the frontend wrappers'
 * names and arg keys exactly. The server compile issue found at ~23:00
 * (`server/src/dispatch.rs` passing `fixtures` by value where `dry_run_lane`
 * takes `&DryRunFixtures`) was fixed by the Rust agent; the server suite is
 * green (91/91) and this spec runs at assembly.
 *
 * The server reads its port, data dir and frontend dir
 * from the environment (main.rs: PORT, LANESRA_DATA_DIR,
 * LANESRA_FRONTEND_DIR); see lane-runner.spec.ts for the setup notes.
 */

const ADMIN_USERNAME = 'admin';
const ADMIN_PASSWORD = 'e2e-runner-admin-1';
const DATA_DIR = process.env.LANESRA_DATA_DIR ?? '/tmp/e2e-data';
const OBJECT_LABEL = 'Widget';
const OBJECT_PLURAL = 'Widgets';

const PUBLISH_LANE_KEY = 'lane-publish-e2e';
const REFUSAL_LANE_KEY = 'lane-publish-refuse-e2e';

/**
 * The dry-run lane: form, record, check, done. The check declares only
 * its pass branch, so P-09 refuses publish (no fail branch) while the
 * dry run still walks the happy path: the check passes on the seed
 * record's Active status and the run ends 'done'.
 */
function publishLaneDefinition(objectKey: string) {
  return {
    format_version: '1.0.0',
    lane: {
      id: PUBLISH_LANE_KEY,
      name: 'Publish Check Lane',
      object_key: objectKey,
      version: 1,
      status: 'draft',
      description: 'The E2E publish-check lane: capture details, post tracking, check the record, done.',
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
              branches: [{ on: 'pass', target: { stage: 'confirm', step: 'review-summary' } }],
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

/**
 * The refusal lane: a check with no fail branch (P-09) and a record step
 * whose Action Type is provisional (P-12). Publish must refuse with both
 * reasons named, not just the first.
 */
function refusalLaneDefinition(objectKey: string) {
  return {
    format_version: '1.0.0',
    lane: {
      id: REFUSAL_LANE_KEY,
      name: 'Refusal Lane',
      object_key: objectKey,
      version: 1,
      status: 'draft',
      description: 'The E2E refusal lane: fails P-09 and P-12 on purpose.',
      run_data: { fields: [] },
      stages: [
        {
          id: 'check',
          name: 'Check',
          steps: [
            {
              id: 'status-check',
              name: 'Status check',
              kind: 'check',
              required: true,
              check: {
                scope: 'anchor',
                message: 'The record is not Active.',
                condition: { field: '$record.status', operator: 'equals', value: 'Active' },
              },
              branches: [{ on: 'pass', target: { stage: 'done', step: 'finish' } }],
            },
            {
              id: 'post-provisional',
              name: 'Post provisional',
              kind: 'record',
              role: 'r',
              required: true,
              writes: [{ action_type: 'Provisional Thing', inputs: {} }],
            },
          ],
        },
        {
          id: 'done',
          name: 'Done',
          steps: [
            {
              id: 'finish',
              name: 'Finish',
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
    // already exists), but a real failure must not be silent.
    // eslint-disable-next-line no-console
    console.log(`[lane-publish seed] ${command} skipped: ${error instanceof Error ? error.message : error}`);
    return undefined;
  }
}

let objectKey = 'widget';
let widgetPublishOneId = '';
let widgetPublishTwoId = '';

test.beforeAll(async ({ request }) => {
  const workspace = await invoke(request, 'workspace_status');
  if (!workspace) {
    const tokenPath = path.join(DATA_DIR, 'setup-token');
    const setupToken = fs.existsSync(tokenPath) ? fs.readFileSync(tokenPath, 'utf8').trim() : undefined;
    await invoke(request, 'first_run_setup', {
      setup: {
        business_name: 'Lane Publish Co',
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

  await invoke(request, 'login', {
    credentials: { username: ADMIN_USERNAME, password: ADMIN_PASSWORD },
  });

  // The widget object, tracking field and Set Tracking Action Type are
  // shared with the lane-runner spec: reuse them when they exist.
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

  // Both lanes stay drafts: publish checks and the dry run are the
  // author's pre-publish tools, and the refusal lane must not publish.
  for (const definition of [publishLaneDefinition(objectKey), refusalLaneDefinition(objectKey)]) {
    await invokeOptional(request, 'import_process_lane', { definitionJson: JSON.stringify(definition) });
  }
  const lanes = await invoke(request, 'list_process_lanes', {});
  for (const key of [PUBLISH_LANE_KEY, REFUSAL_LANE_KEY]) {
    const lane = (lanes as Array<{ lane_key: string }>).find((l) => l.lane_key === key);
    if (!lane) throw new Error(`lane ${key} was not imported by seeding`);
  }

  // The anchor records: Widget publish one runs the dry run, Widget
  // publish two is spare, so a run left active by an interrupted run
  // never blocks the other test (one active run per lane per record).
  const records = await invoke(request, 'list_custom_records', { objectKey });
  for (const [name, slot] of [
    ['Widget publish one', 'one'],
    ['Widget publish two', 'two'],
  ] as const) {
    const existing = (records as Array<{ id: string; primary_name: string }>).find((r) => r.primary_name === name);
    const id = existing
      ? existing.id
      : ((await invoke(request, 'create_custom_record', {
          input: { object_key: objectKey, primary_name: name, status: 'Active', owner_user_id: null, notes: null },
        })) as { id: string }).id;
    if (slot === 'one') widgetPublishOneId = id;
    else widgetPublishTwoId = id;
  }
});

async function loginAsAdmin(page: Page) {
  await page.request.post('/api/invoke/login', {
    data: { credentials: { username: ADMIN_USERNAME, password: ADMIN_PASSWORD } },
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: OBJECT_PLURAL })).toBeVisible();
}

async function openLane(page: Page, laneName: string) {
  await page.getByRole('button', { name: 'Process Lanes' }).click();
  await expect(page.getByRole('heading', { name: 'Process Lanes' })).toBeVisible();
  await page.locator('tr', { hasText: laneName }).click();
  await expect(page.getByRole('heading', { name: laneName })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Publish checks' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Dry run' })).toBeVisible();
}

test('AC-2: the publish-check list renders every check with its named reason', async ({ page }) => {
  await loginAsAdmin(page);
  await openLane(page, 'Publish Check Lane');

  await page.getByRole('button', { name: 'Run publish checks', exact: true }).click();
  const list = page.getByRole('list', { name: 'Publish checks' });
  await expect(list).toContainText('P-09');
  await expect(list).toContainText('tracking-check');
  await expect(list).toContainText('has no fail branch');
  await expect(list).toContainText('refuse');
  await expect(list).toContainText('pass');
  await expect(page.getByText('Cannot publish')).toBeVisible();
});

test('AC-1: a dry run walks the Receiving-style lane and writes nothing', async ({ page }) => {
  await loginAsAdmin(page);
  await openLane(page, 'Publish Check Lane');

  // The record and its runs before the dry run: the dry run must leave
  // both untouched.
  const valuesBefore = await invoke(page.request, 'get_custom_field_values', { entityId: widgetPublishOneId });
  const runsBefore = await invoke(page.request, 'list_lane_runs_for_record', {
    objectKey,
    recordId: widgetPublishOneId,
  });

  await page.getByLabel('Anchor record').selectOption(widgetPublishOneId);
  await page.getByLabel('carrier').fill('ACME');
  await page.getByLabel('Expected outcome').fill('done');
  await page.getByRole('button', { name: 'Run dry run', exact: true }).click();

  const report = page.getByRole('region', { name: 'Dry-run report' });
  await expect(report).toContainText('Stages visited');
  await expect(report).toContainText('Capture');
  await expect(report).toContainText('Enter details');
  await expect(report).toContainText('Tracking check — passed');
  await expect(report).toContainText('pass → Review summary');
  await expect(report).toContainText('Set Tracking');
  await expect(report).toContainText('Simulated only — nothing was written');
  await expect(report).toContainText('End outcome: done');
  await expect(report).toContainText('Matches the expected outcome');

  // Nothing was written: the record's values are unchanged, and no lane
  // run exists on the record.
  const valuesAfter = await invoke(page.request, 'get_custom_field_values', { entityId: widgetPublishOneId });
  expect(valuesAfter).toEqual(valuesBefore);
  expect((valuesAfter as Record<string, unknown>).tracking).not.toBe('ACME');
  const runsAfter = await invoke(page.request, 'list_lane_runs_for_record', {
    objectKey,
    recordId: widgetPublishOneId,
  });
  expect(runsAfter).toEqual(runsBefore);
});

test('AC-2: a publish refusal lists every failed check, not just the first', async ({ page }) => {
  await loginAsAdmin(page);
  await openLane(page, 'Refusal Lane');

  await page.getByRole('button', { name: 'Publish', exact: true }).click();
  await expect(page.getByText('Publish refused')).toBeVisible();
  // Both reasons must appear in the check list itself (not just in the
  // alert banner, which repeats the backend message verbatim). The list
  // is the <ul aria-label="Publish checks">; the section also carries
  // that name, so scope by role, not label.
  const checkList = page.getByRole('list', { name: 'Publish checks' });
  await expect(checkList.getByText(/has no fail branch/)).toBeVisible();
  await expect(checkList.getByText(/provisional Action Type/)).toBeVisible();
  // The lane is still a draft after the refusal.
  await expect(page.getByText('v1 · draft')).toBeVisible();
});
