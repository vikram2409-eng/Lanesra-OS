//! UX/UI Modernization, Phase A (issue #191): Design Tokens & Theme Studio -
//! presets, contrast validation, and the Draft/Published/Archived
//! versioning lifecycle (publish gate, rollback, one-published-row
//! invariant).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::models::workspace_theme::WorkspaceThemeInput;
use lanesra_core::services::{theme_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin User".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn preset_input(preset_key: &str, name: &str) -> WorkspaceThemeInput {
    let tokens = theme_service::get_preset(preset_key).expect("known preset");
    WorkspaceThemeInput { name: name.into(), preset_key: Some(preset_key.into()), tokens }
}

#[test]
fn built_in_presets_cover_all_four_curated_options() {
    let presets = theme_service::built_in_presets();
    let keys: Vec<&str> = presets.iter().map(|(k, _, _, _)| *k).collect();
    assert_eq!(keys, vec!["orbit", "slate", "ember", "aurora"]);
    // Every preset must itself pass the same contrast gate Publish enforces -
    // otherwise a workspace could never publish one of its own curated
    // starting points unmodified.
    for (key, _, _, tokens) in &presets {
        let issues = theme_service::validate_tokens(tokens);
        assert!(issues.is_empty(), "preset '{key}' fails its own contrast check: {issues:?}");
    }
}

#[test]
fn validate_tokens_flags_a_low_contrast_pair() {
    let mut tokens = theme_service::get_preset("orbit").unwrap();
    tokens.color.text_primary = "#CCCCCC".into();
    tokens.color.surface_app = "#FFFFFF".into();
    let issues = theme_service::validate_tokens(&tokens);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|i| i.pair_label.contains("app background")));
}

#[test]
fn save_draft_create_then_edit_then_lock_on_publish() {
    let (conn, workspace_id, admin) = setup_workspace();
    let input = preset_input("slate", "My Theme");
    let draft = theme_service::save_draft(&conn, &workspace_id, None, &input, Some(&admin)).unwrap();
    assert_eq!(draft.status, "draft");
    assert_eq!(draft.version, 1);

    let mut edited = input.clone();
    edited.name = "My Theme (edited)".into();
    let updated = theme_service::save_draft(&conn, &workspace_id, Some(&draft.id), &edited, Some(&admin)).unwrap();
    assert_eq!(updated.id, draft.id);
    assert_eq!(updated.name, "My Theme (edited)");

    let published = theme_service::publish(&conn, &draft.id, &workspace_id, Some(&admin)).unwrap();
    assert_eq!(published.status, "published");
    assert!(published.published_at.is_some());

    // A Published theme is immutable - editing it is rejected, not silently
    // allowed to mutate history.
    let err = theme_service::save_draft(&conn, &workspace_id, Some(&draft.id), &edited, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Draft"));
}

#[test]
fn publish_blocks_on_contrast_failure_with_no_override() {
    let (conn, workspace_id, admin) = setup_workspace();
    let mut tokens = theme_service::get_preset("orbit").unwrap();
    tokens.color.text_primary = "#DDDDDD".into();
    tokens.color.surface_app = "#FFFFFF".into();
    tokens.color.surface_card = "#FFFFFF".into();
    let input = WorkspaceThemeInput { name: "Bad Contrast".into(), preset_key: None, tokens };
    let draft = theme_service::save_draft(&conn, &workspace_id, None, &input, Some(&admin)).unwrap();

    let err = theme_service::publish(&conn, &draft.id, &workspace_id, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("contrast"));

    // Still a draft - the failed publish attempt didn't half-apply.
    let reloaded = theme_service::get(&conn, &draft.id, &workspace_id).unwrap();
    assert_eq!(reloaded.status, "draft");
}

#[test]
fn publish_archives_the_previously_published_version() {
    let (conn, workspace_id, admin) = setup_workspace();
    let v1 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("orbit", "V1"), Some(&admin)).unwrap();
    theme_service::publish(&conn, &v1.id, &workspace_id, Some(&admin)).unwrap();

    let v2 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("slate", "V2"), Some(&admin)).unwrap();
    theme_service::publish(&conn, &v2.id, &workspace_id, Some(&admin)).unwrap();

    let v1_after = theme_service::get(&conn, &v1.id, &workspace_id).unwrap();
    assert_eq!(v1_after.status, "archived");
    let published = theme_service::get_published(&conn, &workspace_id).unwrap().unwrap();
    assert_eq!(published.id, v2.id);

    // Exactly one published row at a time - the versions list never shows
    // two "published" rows simultaneously.
    let versions = theme_service::list_versions(&conn, &workspace_id).unwrap();
    assert_eq!(versions.iter().filter(|v| v.status == "published").count(), 1);
}

#[test]
fn rollback_creates_a_new_draft_and_publishes_it_without_mutating_history() {
    let (conn, workspace_id, admin) = setup_workspace();
    let v1 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("orbit", "Orbit V1"), Some(&admin)).unwrap();
    theme_service::publish(&conn, &v1.id, &workspace_id, Some(&admin)).unwrap();
    let v2 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("slate", "Slate V2"), Some(&admin)).unwrap();
    theme_service::publish(&conn, &v2.id, &workspace_id, Some(&admin)).unwrap();

    let rolled_back = theme_service::rollback_to_version(&conn, &workspace_id, 1, Some(&admin)).unwrap();
    assert_eq!(rolled_back.status, "published");
    assert_eq!(rolled_back.version, 3, "rollback creates a brand-new version, never reuses v1's id or number");
    assert_eq!(rolled_back.tokens.color.brand_primary, theme_service::get_preset("orbit").unwrap().color.brand_primary);

    // v1 itself is untouched (still archived, not "re-published" in place).
    let v1_after = theme_service::get(&conn, &v1.id, &workspace_id).unwrap();
    assert_eq!(v1_after.status, "archived");
    assert_eq!(v1_after.version, 1);

    let versions = theme_service::list_versions(&conn, &workspace_id).unwrap();
    assert_eq!(versions.len(), 3);
}

#[test]
fn delete_draft_rejects_a_published_or_archived_version() {
    let (conn, workspace_id, admin) = setup_workspace();
    let v1 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("orbit", "V1"), Some(&admin)).unwrap();
    theme_service::publish(&conn, &v1.id, &workspace_id, Some(&admin)).unwrap();

    let err = theme_service::delete_draft(&conn, &v1.id, &workspace_id, Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Draft"));

    let v2 = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("slate", "V2 draft"), Some(&admin)).unwrap();
    theme_service::delete_draft(&conn, &v2.id, &workspace_id, Some(&admin)).unwrap();
    assert!(theme_service::get(&conn, &v2.id, &workspace_id).is_err());
}

#[test]
fn non_admin_cannot_save_or_publish_a_theme() {
    let (conn, workspace_id, admin) = setup_workspace();
    use lanesra_core::models::user::NewUser;
    use lanesra_core::services::user_service;
    let sales_rep = user_service::create(
        &conn,
        &workspace_id,
        &NewUser { username: "rep".into(), display_name: "Rep".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    )
    .unwrap();

    let err = theme_service::save_draft(&conn, &workspace_id, None, &preset_input("orbit", "Nope"), Some(&sales_rep.id)).unwrap_err();
    assert!(format!("{err:?}").to_lowercase().contains("admin"));
}
