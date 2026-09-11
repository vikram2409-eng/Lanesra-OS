-- AI & Agentic Layer, Phase 7f: an optional OTLP collector endpoint a
-- run's trace (ai_orchestration_service::run_to_otlp_json) can be pushed
-- to on demand - its own admin dial on ai_settings, the same "own field,
-- own setter" shape daily_token_budget (migration 0041) already uses.
ALTER TABLE ai_settings ADD COLUMN otlp_endpoint TEXT;
