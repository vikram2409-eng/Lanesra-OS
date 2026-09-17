-- Voice-First Mode, PR 1 of 3 (spec: "Lanesra OS Voice-First Mode" v0.1 Voice
-- Assist + v0.2 Voice Actions). A secure voice interaction channel into the
-- existing object model, Access Control v1, Business Rules, Workflow
-- Automation and audit trail - not a parallel automation engine. See
-- services::voice_execution_service's own doc comment for the non-negotiable
-- design principle this schema exists to serve: every voice-triggered write
-- goes through the exact same service functions the UI/API already call.
--
-- Nine tables, in the order a command actually flows through them:
-- settings/policy (config) -> session (unlock) -> command (one utterance)
-- -> resolution (what it refers to) -> action_plan (what it proposes)
-- -> confirmation (user's response) -> execution (what actually ran)
-- -> provider_profiles (speech provider config, orthogonal to the flow).

CREATE TABLE voice_user_settings (
    user_id TEXT PRIMARY KEY REFERENCES users(id),
    enabled INTEGER NOT NULL DEFAULT 0,
    pin_hash TEXT,
    pin_set_at TEXT,
    failed_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TEXT,
    response_channel TEXT NOT NULL DEFAULT 'voice_and_text' CHECK (response_channel IN ('text_only', 'voice_and_text')),
    spoken_detail TEXT NOT NULL DEFAULT 'normal' CHECK (spoken_detail IN ('short', 'normal', 'detailed')),
    auto_speak_confirmations INTEGER NOT NULL DEFAULT 0,
    quiet_mode INTEGER NOT NULL DEFAULT 0,
    unlock_duration_minutes INTEGER NOT NULL DEFAULT 15,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Voice's own capability+risk policy, resolved per Access Role exactly the
-- way access_service resolves object capability grants (broadest-wins union
-- across a user's roles) - see voice_policy_service::voice_capability_allows.
-- A NULL access_role_id row is the workspace-wide default, used as a
-- fallback for any role without its own row (same '*'-fallback idea
-- access_role_grants already uses for object_key).
CREATE TABLE voice_policy_bindings (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    access_role_id TEXT REFERENCES access_roles(id),
    can_use_voice INTEGER NOT NULL DEFAULT 0,
    can_search INTEGER NOT NULL DEFAULT 0,
    can_create INTEGER NOT NULL DEFAULT 0,
    can_update INTEGER NOT NULL DEFAULT 0,
    can_act INTEGER NOT NULL DEFAULT 0,
    can_bulk_act INTEGER NOT NULL DEFAULT 0,
    can_external_act INTEGER NOT NULL DEFAULT 0,
    can_use_agents INTEGER NOT NULL DEFAULT 0,
    max_action_level TEXT NOT NULL DEFAULT 'ask_only' CHECK (max_action_level IN ('ask_only', 'capture', 'act_with_confirmation', 'act')),
    processing_boundary TEXT NOT NULL DEFAULT 'cloud' CHECK (processing_boundary IN ('cloud', 'approved_private', 'local_only')),
    confidence_thresholds_json TEXT NOT NULL DEFAULT '{}',
    max_unlock_minutes INTEGER NOT NULL DEFAULT 30,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (workspace_id, access_role_id)
);
CREATE INDEX idx_voice_policy_bindings_workspace ON voice_policy_bindings(workspace_id);

CREATE TABLE voice_sessions (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    state TEXT NOT NULL DEFAULT 'idle' CHECK (state IN (
        'locked', 'ready', 'listening', 'processing', 'needs_clarification',
        'awaiting_confirmation', 'awaiting_approval', 'speaking', 'idle', 'expired'
    )),
    unlocked_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    context_object_key TEXT,
    context_record_id TEXT,
    conversation_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_voice_sessions_user ON voice_sessions(user_id);

CREATE TABLE voice_commands (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES voice_sessions(id),
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    transcript TEXT NOT NULL,
    language TEXT NOT NULL DEFAULT 'en-US',
    speech_confidence REAL,
    status TEXT NOT NULL DEFAULT 'received' CHECK (status IN (
        'received', 'resolved', 'needs_clarification', 'planned', 'executed', 'failed', 'cancelled'
    )),
    correlation_id TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_voice_commands_session ON voice_commands(session_id);
CREATE INDEX idx_voice_commands_user ON voice_commands(user_id);
CREATE INDEX idx_voice_commands_correlation ON voice_commands(correlation_id);

CREATE TABLE voice_resolutions (
    id TEXT PRIMARY KEY,
    command_id TEXT NOT NULL REFERENCES voice_commands(id),
    intent TEXT NOT NULL,
    object_key TEXT,
    record_reference_text TEXT,
    resolved_record_id TEXT,
    intent_confidence REAL NOT NULL,
    entity_confidence REAL,
    candidates_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL
);
CREATE INDEX idx_voice_resolutions_command ON voice_resolutions(command_id);

CREATE TABLE voice_action_plans (
    id TEXT PRIMARY KEY,
    command_id TEXT NOT NULL REFERENCES voice_commands(id),
    plan_json TEXT NOT NULL,
    risk TEXT NOT NULL DEFAULT 'none' CHECK (risk IN ('none', 'low', 'medium', 'high', 'critical', 'blocked')),
    confirmation_required INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN (
        'draft', 'awaiting_confirmation', 'awaiting_approval', 'confirmed',
        'rejected', 'executing', 'succeeded', 'partially_failed', 'failed'
    )),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_voice_action_plans_command ON voice_action_plans(command_id);
CREATE INDEX idx_voice_action_plans_status ON voice_action_plans(status);

CREATE TABLE voice_confirmations (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES voice_action_plans(id),
    method TEXT NOT NULL CHECK (method IN ('voice', 'tap', 'auto')),
    outcome TEXT NOT NULL CHECK (outcome IN ('confirmed', 'rejected', 'edited')),
    edited_plan_json TEXT,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL
);
CREATE INDEX idx_voice_confirmations_plan ON voice_confirmations(plan_id);

CREATE TABLE voice_executions (
    id TEXT PRIMARY KEY,
    plan_id TEXT NOT NULL REFERENCES voice_action_plans(id),
    step_index INTEGER NOT NULL,
    entity_type TEXT NOT NULL,
    entity_id TEXT,
    action TEXT NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('ok', 'error')),
    error_message TEXT,
    undo_token TEXT,
    undone_at TEXT,
    correlation_id TEXT NOT NULL,
    executed_at TEXT NOT NULL
);
CREATE INDEX idx_voice_executions_plan ON voice_executions(plan_id);
CREATE INDEX idx_voice_executions_correlation ON voice_executions(correlation_id);
CREATE INDEX idx_voice_executions_undo_token ON voice_executions(undo_token);

-- Provider abstraction, same shape as ai_provider_service's own config
-- table: real, workspace-scoped rows an admin can see/health-check, not a
-- hardcoded constant. PR 1 seeds exactly one real adapter (browser Web
-- Speech API) per workspace; 'cloud'/'local' rows are placeholders an admin
-- can add once a real adapter for them ships (not this PR - see the
-- roadmap's explicit "Local Voice Runtime" gap).
CREATE TABLE voice_provider_profiles (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL REFERENCES workspaces(id),
    name TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('web_speech', 'cloud', 'local')),
    privacy_class TEXT NOT NULL CHECK (privacy_class IN ('external', 'private', 'local')),
    is_default INTEGER NOT NULL DEFAULT 0,
    last_health_check_at TEXT,
    last_health_status TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_voice_provider_profiles_workspace ON voice_provider_profiles(workspace_id);

INSERT INTO voice_provider_profiles (id, workspace_id, name, kind, privacy_class, is_default, created_at, updated_at)
SELECT lower(hex(randomblob(16))), id, 'Browser (Web Speech API)', 'web_speech', 'external', 1, created_at, created_at
FROM workspaces;

-- Bootstrap a workspace-default policy binding (access_role_id NULL) so
-- Voice Mode is immediately configurable without an empty-policy cliff -
-- deliberately Ask Only / can_use_voice=0 by default: an Administrator must
-- opt each Access Role into Voice, mirroring how Access Control v1 itself
-- never silently widens access on upgrade.
INSERT INTO voice_policy_bindings (id, workspace_id, access_role_id, can_use_voice, can_search, can_create, can_update, can_act, can_bulk_act, can_external_act, can_use_agents, max_action_level, processing_boundary, confidence_thresholds_json, max_unlock_minutes, created_at, updated_at)
SELECT lower(hex(randomblob(16))), id, NULL, 0, 1, 0, 0, 0, 0, 0, 0, 'ask_only', 'cloud', '{}', 15, created_at, created_at
FROM workspaces;

-- The "Full Access" system Access Role (seeded by 0056) gets full Voice
-- capabilities immediately, matching how it already gets every object
-- capability - an Administrator can use Voice Mode end-to-end out of the
-- box to evaluate it, without editing policy first.
INSERT INTO voice_policy_bindings (id, workspace_id, access_role_id, can_use_voice, can_search, can_create, can_update, can_act, can_bulk_act, can_external_act, can_use_agents, max_action_level, processing_boundary, confidence_thresholds_json, max_unlock_minutes, created_at, updated_at)
SELECT lower(hex(randomblob(16))), ar.workspace_id, ar.id, 1, 1, 1, 1, 1, 1, 1, 1, 'act_with_confirmation', 'cloud', '{}', 30, ar.created_at, ar.created_at
FROM access_roles ar WHERE ar.name = 'Full Access';
