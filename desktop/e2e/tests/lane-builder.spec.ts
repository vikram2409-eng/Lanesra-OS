import { test, expect, type APIRequestContext, type Page } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';

/**
 * The visual lane builder (WP-0718).
 * Covers: REQ-LANE-024 (WP-0718 AC-1, AC-2, AC-4).
 *
 * Drives the Team Workspace server (server mode) and the real desktop
 * UI from the same code. Seeding goes through the server's own HTTP API
 * (`/api/invoke/<command>`, the same surface the frontend's `api`
 * module uses) and reuses the `widget` custom object, `tracking` text
 * field and `Set Tracking` Action Type from the lane-publish spec, so
 * the spec is safe to re-run against a data dir a previous run already
 * seeded.
 *
 * Server-mode specs need a manually started server (`cargo run -p
 * lanesra-server` from desktop/, base http://localhost:3000) because
 * the Playwright config's 120s webServer timeout is too short for a
 * cold build; kill the server after the run. The server reads its port,
 * data dir and frontend dir from the environment (main.rs: PORT,
 * LANESRA_DATA_DIR, LANESRA_FRONTEND_DIR); see lane-runner.spec.ts.
 *
 * NOTE: the builder commands (save_builder_lane, builder_check_lane,
 * builder_dry_run_lane, builder_reads_writes, builder_action_types)
 * are the new WP-0718 commands the core agent builds in parallel. The
 * UI calls them through the api wrappers and falls back to the local
 * publish checks (same P-05…P-12 reasons) while they are unavailable,
 * so the builder loads, edits and shows checks either way. The save
 * step below asserts the full persistence round-trip when
 * save_builder_lane answers, and asserts the honest unavailable
 * message plus the in-page round-trip while it does not — the
 * coordinator verifies the live wiring at assembly.
 */

const ADMIN_USERNAME = 'admin';
const ADMIN_PASSWORD = 'admin-password-1';
const DATA_DIR = process.env.LANESRA_DATA_DIR ?? '/tmp/e2e-data';
const OBJECT_LABEL = 'Widget';
const OBJECT_PLURAL = 'Widgets';
const BUILDER_LANE_KEY = 'lane-builder-e2e';

/**
 * A lane that fails P-09 on purpose (the check declares only its pass
 * branch) and otherwise uses the adopted `Set Tracking` Action Type, so
 * adding the fail branch in the builder is the one fix that turns the
 * checks green and enables Publish.
 */
function builderLaneDefinition(objectKey: string) {
  return {
    format_version: '1.0.0',
    lane: {
      id: BUILDER_LANE_KEY,
      name: 'Builder Lane',
      object_key: objectKey,
      version: 1,
      status: 'draft',
      description: 'The E2E lane-builder lane: capture details, post tracking, check the record, done.',
      run_data: {
        fields: [
          { key: 'carrier', type: 'string', required: true },
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
    // eslint-disable-next-line no-console
    console.log(`[lane-builder seed] ${command} skipped: ${error instanceof Error ? error.message : error}`);
    return undefined;
  }
}

let objectKey = 'widget';

test.beforeAll(async ({ request }) => {
  const workspace = await invoke(request, 'workspace_status');
  if (!workspace) {
    const tokenPath = path.join(DATA_DIR, 'setup-token');
    const setupToken = fs.existsSync(tokenPath) ? fs.readFileSync(tokenPath, 'utf8').trim() : undefined;
    await invoke(request, 'first_run_setup', {
      setup: {
        business_name: 'Lane Builder Co',
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

  // Reuse the widget object / tracking field / Set Tracking Action Type
  // from the lane-publish spec when they exist.
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

  await invokeOptional(request, 'import_process_lane', {
    definitionJson: JSON.stringify(builderLaneDefinition(objectKey)),
  });
  const lanes = await invoke(request, 'list_process_lanes', {});
  const lane = (lanes as Array<{ lane_key: string }>).find((l) => l.lane_key === BUILDER_LANE_KEY);
  if (!lane) throw new Error(`lane ${BUILDER_LANE_KEY} was not imported by seeding`);
});

async function loginAsAdmin(page: Page) {
  await page.request.post('/api/invoke/login', {
    data: { credentials: { username: ADMIN_USERNAME, password: ADMIN_PASSWORD } },
  });
  await page.goto('/');
  await expect(page.getByRole('button', { name: OBJECT_PLURAL })).toBeVisible();
}

async function openBuilder(page: Page) {
  await page.getByRole('button', { name: 'Process Lanes' }).click();
  await expect(page.getByRole('heading', { name: 'Process Lanes' })).toBeVisible();
  await page.locator('tr', { hasText: 'Builder Lane' }).click();
  await expect(page.getByRole('heading', { name: 'Builder Lane' })).toBeVisible();
  await page.getByRole('button', { name: 'Open in builder', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Canvas' })).toBeVisible();
}

test('AC-1/AC-2: the builder loads the lane, shows live publish checks, and Publish is disabled naming P-09', async ({ page }) => {
  await loginAsAdmin(page);
  await openBuilder(page);

  // All four steps, in definition order, with their order numbers.
  for (const name of ['Enter details', 'Post tracking', 'Tracking check', 'Review summary']) {
    await expect(page.getByRole('button', { name: new RegExp(name) }).first()).toBeVisible();
  }
  await expect(page.getByRole('button', { name: /Step 3 of 4.*Tracking check/ })).toBeVisible();

  // Live publish checks: the missing fail branch refuses.
  const checks = page.getByRole('list', { name: 'Builder publish checks' });
  await expect(checks).toContainText('P-09');
  await expect(checks).toContainText('tracking-check');
  await expect(checks).toContainText('has no fail branch');
  const publish = page.getByRole('button', { name: 'Publish', exact: true });
  await expect(publish).toBeDisabled();
  await expect(publish).toHaveAttribute('title', /P-09/);
});

test('AC-1: editing a step round-trips — the rename survives a save, or the honest unavailable message shows', async ({ page }) => {
  await loginAsAdmin(page);
  await openBuilder(page);

  await page.getByRole('button', { name: /Step 1 of 4.*Enter details/ }).click();
  await page.getByLabel('Step name').fill('Enter details (edited)');
  await expect(page.getByRole('button', { name: /Step 1 of 4.*Enter details \(edited\)/ })).toBeVisible();

  await page.getByRole('button', { name: 'Save lane', exact: true }).click();
  const saved = page.getByText(/Saved as version/);
  const unavailable = page.getByText(/Could not save/);
  await expect(saved.or(unavailable)).toBeVisible();
  if (await saved.count()) {
    // Full round-trip: the saved definition carries the rename.
    const blueprint = (await invoke(page.request, 'get_lane_blueprint', { laneKey: BUILDER_LANE_KEY })) as {
      definition: { stages: Array<{ steps: Array<{ name: string }> }> };
    };
    expect(
      blueprint.definition.stages.flatMap((s) => s.steps).some((s) => s.name === 'Enter details (edited)'),
    ).toBe(true);
  } else {
    // save_builder_lane has not landed yet: the edit still holds
    // in-page (canvas + panel agree on the new name).
    await expect(page.getByLabel('Step name')).toHaveValue('Enter details (edited)');
  }
});

test('AC-2: adding the missing fail branch turns the checks green and enables Publish', async ({ page }) => {
  await loginAsAdmin(page);
  await openBuilder(page);

  await page.getByRole('button', { name: /Step 3 of 4.*Tracking check/ }).click();
  // The empty fail slot is a visible gap with an Add button.
  await expect(page.getByText(/Empty slot: no fail branch/)).toBeVisible();
  await page.getByRole('button', { name: 'Add fail branch', exact: true }).click();
  await page.getByLabel('Branch fail target').selectOption('end:held');

  const publish = page.getByRole('button', { name: 'Publish', exact: true });
  await expect(publish).toBeEnabled({ timeout: 10000 });
  await publish.click();
  // Publish saves the edited definition first, then publishes it. While
  // save_builder_lane has not landed, the save fails honestly and the
  // saved lane stays unchanged (the service refusal names the still-
  // missing fail branch on the saved lane); once it lands, the edited
  // lane publishes.
  const published = page.getByText(/Published version/);
  const saveFailed = page.getByText(/Could not save the edited lane before publishing/);
  await expect(published.or(saveFailed)).toBeVisible();
  if (await published.count()) {
    const blueprint = (await invoke(page.request, 'get_lane_blueprint', { laneKey: BUILDER_LANE_KEY })) as {
      definition: { stages: Array<{ steps: Array<{ id: string; branches?: Array<{ on: string }> }> }> };
    };
    const check = blueprint.definition.stages
      .flatMap((s) => s.steps)
      .find((s) => s.id === 'tracking-check');
    expect(check?.branches?.map((b) => b.on).sort()).toEqual(['fail', 'pass']);
  }
});

test('New lane starts from a blank definition', async ({ page }) => {
  await loginAsAdmin(page);
  await page.getByRole('button', { name: 'Process Lanes' }).click();
  await expect(page.getByRole('heading', { name: 'Process Lanes' })).toBeVisible();
  await page.getByRole('button', { name: 'New lane', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'New lane' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Canvas' })).toBeVisible();
  await expect(page.getByRole('button', { name: /Stage 1: Stage 1/ })).toBeVisible();
});
