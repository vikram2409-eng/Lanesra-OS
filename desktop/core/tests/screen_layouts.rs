//! Screen/App Builder Phase 1: multiple named layouts per object, tabs of
//! drag-ordered field sections, role-based assignment with a required
//! default fallback, and Draft -> Preview -> Publish. Mirrors the online
//! demo's equivalent tests conceptually, but exercises the one thing the
//! demo can't: resolving a layout against a real signed-in user's roles.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::screen_layout::{LayoutSection, LayoutTab, LayoutTabs, ScreenLayoutInput, ScreenLayoutUpdate, SectionField};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{screen_layout_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Screen Layouts Co".into(),
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

fn sales_user(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    user_service::create(
        conn, ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(admin),
    )
    .unwrap()
    .id
}

fn layout_input(name: &str, initial_fields: &[&str]) -> ScreenLayoutInput {
    ScreenLayoutInput {
        entity_type: "Company".into(),
        name: name.into(),
        initial_fields: initial_fields.iter().map(|s| s.to_string()).collect(),
    }
}

fn one_field_draft(field: &str) -> LayoutTabs {
    LayoutTabs {
        tabs: vec![LayoutTab {
            id: "t1".into(),
            title: "Details".into(),
            sections: vec![LayoutSection { id: "s1".into(), title: "Details".into(), columns: 2, fields: vec![field.into()] }],
            related: vec![],
        }],
    }
}

/// The key list a section's fields hold, in order - lets tests assert on
/// field placement without spelling out each `SectionField`'s full_width.
fn field_keys(section: &LayoutSection) -> Vec<String> {
    section.fields.iter().map(|f| f.key.clone()).collect()
}

#[test]
fn listing_auto_provisions_a_default_layout() {
    let (conn, ws, _admin) = setup_workspace();
    let layouts = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].name, "Default");
    assert!(layouts[0].is_default);
    assert!(layouts[0].published.is_none(), "auto-provisioned layout must start unpublished");
}

#[test]
fn a_new_layout_is_not_default_and_the_first_ever_layout_is() {
    let (conn, ws, admin) = setup_workspace();
    // Force auto-provisioning of the initial Default layout first.
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();

    let second = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &["industry"]), Some(&admin)).unwrap();
    assert!(!second.is_default);
    assert_eq!(field_keys(&second.draft.tabs[0].sections[0]), vec!["industry".to_string()]);
}

#[test]
fn the_first_layout_created_for_an_entity_becomes_the_default() {
    let (conn, ws, admin) = setup_workspace();
    // No list_layouts call yet - so nothing has auto-provisioned a Default.
    let only = screen_layout_service::create_layout(&conn, &ws, &layout_input("Only layout", &[]), Some(&admin)).unwrap();
    assert!(only.is_default);
}

#[test]
fn a_non_administrator_cannot_create_a_layout() {
    let (conn, ws, admin) = setup_workspace();
    let sam = sales_user(&conn, &ws, &admin);
    let result = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&sam));
    assert!(result.is_err());
}

#[test]
fn publish_unpublish_and_revert_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);

    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_field_draft("industry") };
    let updated = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert!(updated.published.is_none());

    let published = screen_layout_service::publish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(published.published, Some(published.draft.clone()));

    // Edit the draft further without publishing - published stays as-is.
    let update2 = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: one_field_draft("city") };
    let edited = screen_layout_service::update_layout(&conn, &layout.id, &update2, Some(&admin)).unwrap();
    assert_ne!(edited.draft, edited.published.clone().unwrap());

    let reverted = screen_layout_service::revert_layout_draft(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(reverted.draft, reverted.published.clone().unwrap());

    let unpublished = screen_layout_service::unpublish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert!(unpublished.published.is_none());
}

#[test]
fn make_default_moves_the_flag_and_only_one_layout_is_ever_default() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let other = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    assert!(!other.is_default);

    let promoted = screen_layout_service::make_default(&conn, &other.id, Some(&admin)).unwrap();
    assert!(promoted.is_default);

    let old_default = screen_layout_service::get_layout(&conn, &default_layout.id).unwrap();
    assert!(!old_default.is_default);

    let all = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(all.iter().filter(|l| l.is_default).count(), 1);
}

#[test]
fn the_default_layout_cannot_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let result = screen_layout_service::delete_layout(&conn, &default_layout.id, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn a_non_default_layout_can_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let other = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    screen_layout_service::delete_layout(&conn, &other.id, Some(&admin)).unwrap();
    let remaining = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn resolve_effective_layout_returns_none_when_nothing_is_published() {
    let (conn, ws, _admin) = setup_workspace();
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", None).unwrap();
    assert!(effective.is_none());
}

#[test]
fn resolve_effective_layout_falls_back_to_the_published_default_for_an_unclaimed_role() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let update = ScreenLayoutUpdate { name: default_layout.name.clone(), roles: vec![], draft: one_field_draft("industry") };
    screen_layout_service::update_layout(&conn, &default_layout.id, &update, Some(&admin)).unwrap();
    screen_layout_service::publish_layout(&conn, &default_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(field_keys(&effective.unwrap().tabs[0].sections[0]), vec!["industry".to_string()]);
}

#[test]
fn resolve_effective_layout_prefers_a_published_layout_matching_the_actors_role() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_layout = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    let update = ScreenLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_field_draft("city") };
    screen_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    screen_layout_service::publish_layout(&conn, &sales_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(field_keys(&effective.unwrap().tabs[0].sections[0]), vec!["city".to_string()]);

    // The Administrator (no Sales role) is unaffected - falls back to
    // Default, which was never published, so still None.
    let admin_effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(admin_effective.is_none());
}

#[test]
fn an_unpublished_role_matching_layout_is_never_used_live() {
    let (conn, ws, admin) = setup_workspace();
    screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_layout = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &[]), Some(&admin)).unwrap();
    let update = ScreenLayoutUpdate { name: sales_layout.name.clone(), roles: vec!["Sales".into()], draft: one_field_draft("city") };
    screen_layout_service::update_layout(&conn, &sales_layout.id, &update, Some(&admin)).unwrap();
    // Deliberately never published.

    let sam = sales_user(&conn, &ws, &admin);
    let effective = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert!(effective.is_none(), "a draft-only layout must never affect the live form");
}

#[test]
fn an_invalid_entity_type_is_rejected() {
    let (conn, ws, _admin) = setup_workspace();
    let result = screen_layout_service::list_layouts(&conn, &ws, "NotARealEntity");
    assert!(result.is_err());
}

// --- Screen/App Builder Phase 2: multi-column sections ---

#[test]
fn a_newly_created_layout_seeds_a_two_column_section_with_no_full_width_fields() {
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::create_layout(&conn, &ws, &layout_input("Sales layout", &["industry", "city"]), Some(&admin)).unwrap();
    let section = &layout.draft.tabs[0].sections[0];
    assert_eq!(section.columns, 2);
    assert!(section.fields.iter().all(|f| !f.full_width), "fresh fields default to a single column");
}

#[test]
fn a_sections_column_count_and_each_fields_full_width_flag_round_trip_through_save() {
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);

    let draft = LayoutTabs {
        tabs: vec![LayoutTab {
            id: "t1".into(),
            title: "Details".into(),
            sections: vec![LayoutSection {
                id: "s1".into(),
                title: "Details".into(),
                columns: 3,
                fields: vec![
                    SectionField { key: "name".into(), full_width: true },
                    SectionField { key: "industry".into(), full_width: false },
                ],
            }],
            related: vec![],
        }],
    };
    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
    let saved = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();

    let section = &saved.draft.tabs[0].sections[0];
    assert_eq!(section.columns, 3);
    assert_eq!(section.fields[0], SectionField { key: "name".into(), full_width: true });
    assert_eq!(section.fields[1], SectionField { key: "industry".into(), full_width: false });

    // And it's still there after a fresh read, not just the mutation's own response.
    let reloaded = screen_layout_service::get_layout(&conn, &layout.id).unwrap();
    assert_eq!(reloaded.draft.tabs[0].sections[0].columns, 3);
}

#[test]
fn a_phase_1_layout_saved_before_columns_existed_still_loads_with_the_old_default() {
    // Simulates a draft persisted by the Phase 1 code: "fields" as bare
    // key strings, no "columns" key on the section at all.
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let legacy_draft_json = r#"{"tabs":[{"id":"t1","title":"Details","sections":[{"id":"s1","title":"Details","fields":["industry","city"]}]}]}"#;

    // Go around the service (which only ever serializes the new shape) to
    // write the legacy JSON directly into the draft column, then read it
    // back through the ordinary service path.
    conn.execute("UPDATE screen_layouts SET draft_json = ?1 WHERE id = ?2", rusqlite::params![legacy_draft_json, layout.id]).unwrap();

    let reloaded = screen_layout_service::get_layout(&conn, &layout.id).unwrap();
    let section = &reloaded.draft.tabs[0].sections[0];
    assert_eq!(section.columns, 2, "a section with no stored columns falls back to the Phase 1 fixed width");
    assert_eq!(field_keys(section), vec!["industry".to_string(), "city".to_string()]);
    assert!(section.fields.iter().all(|f| !f.full_width));
    assert!(reloaded.draft.tabs[0].related.is_empty(), "a tab with no stored 'related' key falls back to none placed");

    // And it now round-trips as the new shape once saved again.
    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: reloaded.draft.clone() };
    let resaved = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert_eq!(resaved.draft, reloaded.draft);
}

// --- Screen/App Builder Phase 3: related-list tab placement ---

#[test]
fn a_tabs_related_list_keys_round_trip_through_save() {
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);

    let draft = LayoutTabs {
        tabs: vec![
            LayoutTab { id: "t1".into(), title: "Details".into(), sections: vec![], related: vec!["primary_contacts".into()] },
            LayoutTab { id: "t2".into(), title: "History".into(), sections: vec![], related: vec!["opportunities".into(), "quotes".into()] },
        ],
    };
    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
    let saved = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();

    assert_eq!(saved.draft.tabs[0].related, vec!["primary_contacts".to_string()]);
    assert_eq!(saved.draft.tabs[1].related, vec!["opportunities".to_string(), "quotes".to_string()]);

    // A tab can carry only a related list and no field sections at all -
    // it isn't required to have both.
    assert!(saved.draft.tabs[0].sections.is_empty());

    let reloaded = screen_layout_service::get_layout(&conn, &layout.id).unwrap();
    assert_eq!(reloaded.draft, saved.draft);
}

#[test]
fn a_phase_2_layout_saved_before_related_lists_existed_still_loads_with_none_placed() {
    // Simulates a draft persisted by the Phase 1/2 code: sections in the
    // current shape, but no "related" key on the tab at all.
    let (conn, ws, admin) = setup_workspace();
    let layout = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let legacy_draft_json = r#"{"tabs":[{"id":"t1","title":"Details","sections":[{"id":"s1","title":"Details","columns":2,"fields":[{"key":"industry","full_width":false}]}]}]}"#;
    conn.execute("UPDATE screen_layouts SET draft_json = ?1 WHERE id = ?2", rusqlite::params![legacy_draft_json, layout.id]).unwrap();

    let reloaded = screen_layout_service::get_layout(&conn, &layout.id).unwrap();
    assert!(reloaded.draft.tabs[0].related.is_empty());
    assert_eq!(field_keys(&reloaded.draft.tabs[0].sections[0]), vec!["industry".to_string()]);

    let update = ScreenLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: reloaded.draft.clone() };
    let resaved = screen_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert_eq!(resaved.draft, reloaded.draft);
}
