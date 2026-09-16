-- Enterprise Access Foundation, Phase 1 continued: Ownership Mode (spec
-- §2.1) for Custom Objects. Custom objects already have a per-object-type
-- metadata row (custom_object_definitions), unlike built-in objects, so
-- this is a real column here rather than the static Rust map used for
-- built-ins (see ownership_service::BUILTIN_OWNERSHIP_MODES) - a custom
-- object genuinely can be configured differently per workspace, a built-in
-- object's ownership shape is fixed by the platform.
ALTER TABLE custom_object_definitions ADD COLUMN ownership_mode TEXT NOT NULL DEFAULT 'USER_TEAM_OWNED' CHECK (ownership_mode IN ('USER_TEAM_OWNED', 'ORG_OWNED', 'PARENT_CONTROLLED', 'SYSTEM_OWNED'));
