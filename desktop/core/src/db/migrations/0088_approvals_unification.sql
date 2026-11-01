-- Migration 0088: every pause for a person goes through ai_approvals (WP-0150)
-- Work package: WP-0150
-- Change: data (three nullable columns, then a backfill of pending pauses)
-- Data touched: ai_approvals gains one pending row per pipeline run waiting for approval and per voice plan waiting for approval; those runs and plans are linked to their row
-- Re-run safe: no (ALTER TABLE ADD COLUMN)
--
-- A pipeline run paused for approval used to wait only on its own row
-- (ai_agent_runs.status = 'awaiting_approval'), invisible to the approvals
-- inbox; a voice plan that needed approval had no way to be decided at all.
-- Both now hold a pending ai_approvals row, and the inbox decides them the
-- same way it decides an execution-graph Approval node.
--
-- ai_approvals.consumed_at: an approved tool call (subject_type
-- 'tool_call') lets that exact call run once; the run that uses it sets
-- this so the grant cannot be used twice.
ALTER TABLE ai_agent_runs ADD COLUMN pending_approval_id TEXT REFERENCES ai_approvals(id);
ALTER TABLE voice_action_plans ADD COLUMN approval_id TEXT REFERENCES ai_approvals(id);
ALTER TABLE ai_approvals ADD COLUMN consumed_at TEXT;

-- Pipeline runs already waiting when this release is installed.
INSERT INTO ai_approvals (id, workspace_id, subject_type, subject_id, proposal_json, status, requested_by, created_at)
SELECT lower(hex(randomblob(16))), r.workspace_id, 'pipeline_run', r.id,
       json_object('run_id', r.id, 'pipeline_id', r.target_id, 'step_order', r.paused_at_step_order,
                   'proposed_output', r.resume_previous_output, 'migrated', 1),
       'pending', NULL, r.started_at
FROM ai_agent_runs r
WHERE r.status = 'awaiting_approval';

UPDATE ai_agent_runs
SET pending_approval_id = (
    SELECT a.id FROM ai_approvals a
    WHERE a.subject_type = 'pipeline_run' AND a.subject_id = ai_agent_runs.id AND a.status = 'pending'
)
WHERE status = 'awaiting_approval';

-- Voice plans already waiting for approval.
INSERT INTO ai_approvals (id, workspace_id, subject_type, subject_id, proposal_json, status, requested_by, created_at)
SELECT lower(hex(randomblob(16))), c.workspace_id, 'voice_plan', p.id,
       json_object('plan_id', p.id, 'command_id', p.command_id, 'transcript', c.transcript,
                   'risk', p.risk, 'plan', json(p.plan_json), 'migrated', 1),
       'pending', c.user_id, p.created_at
FROM voice_action_plans p
JOIN voice_commands c ON c.id = p.command_id
WHERE p.status = 'awaiting_approval';

UPDATE voice_action_plans
SET approval_id = (
    SELECT a.id FROM ai_approvals a
    WHERE a.subject_type = 'voice_plan' AND a.subject_id = voice_action_plans.id AND a.status = 'pending'
)
WHERE status = 'awaiting_approval';
