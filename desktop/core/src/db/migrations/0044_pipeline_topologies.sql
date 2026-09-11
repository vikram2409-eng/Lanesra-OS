-- AI & Agentic Layer, Phase 7d: two more orchestration topologies on top
-- of Phase 6b's Pipeline - 'consensus' (every step but the last runs
-- independently against the same trigger input; the last step
-- synthesizes all of them via a new {{candidate_outputs}} placeholder)
-- and 'peer_review' (exactly two steps - a drafter and a reviewer -
-- looping until the reviewer approves or a round cap is hit). The
-- existing 'sequential' behavior (migration 0040) is completely
-- unchanged and stays the default. See
-- core::services::ai_orchestration_service::run_internal.
ALTER TABLE ai_agent_pipelines ADD COLUMN topology TEXT NOT NULL DEFAULT 'sequential';
