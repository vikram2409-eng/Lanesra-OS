//! Screen Builder 2.0 (issue #195, 5a): a page composer alongside (not
//! instead of) Screen/App Builder's screen_layouts system - see
//! page_layout.rs's own doc comment and the migration's header comment.
//! Mirrors screen_layouts.rs's own test shape for the shared draft/
//! published/default/role lifecycle, plus two things unique to this
//! layer: component-type validation, and an explicit parity test proving
//! the two systems never touch each other's data.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::page_layout::{NodeLayout, PageDefinition, PageLayoutInput, PageLayoutUpdate, PageNode};
use lanesra_core::models::screen_layout::{LayoutTab, LayoutTabs, ScreenLayoutUpdate};
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::page_layout_service::{self, COMPONENT_TYPES, CONTAINER_COMPONENT_TYPES};
use lanesra_core::services::{screen_layout_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Page Builder Co".into(),
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

fn layout_input(name: &str) -> PageLayoutInput {
    PageLayoutInput { entity_type: "Company".into(), name: name.into() }
}

fn leaf(component_type: &str, column_span: u8) -> PageNode {
    PageNode {
        id: format!("n-{component_type}"),
        component_type: component_type.into(),
        config: serde_json::json!({}),
        children: vec![],
        layout: NodeLayout { column_span, tablet_column_span: None, mobile_column_span: None, order: 0 },
    }
}

fn container(component_type: &str, children: Vec<PageNode>) -> PageNode {
    PageNode {
        id: format!("n-{component_type}"),
        component_type: component_type.into(),
        config: serde_json::json!({}),
        children,
        layout: NodeLayout { column_span: 12, tablet_column_span: None, mobile_column_span: None, order: 0 },
    }
}

#[test]
fn listing_auto_provisions_a_default_page() {
    let (conn, ws, _admin) = setup_workspace();
    let layouts = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(layouts.len(), 1);
    assert_eq!(layouts[0].name, "Default");
    assert!(layouts[0].is_default);
    assert!(layouts[0].published.is_none(), "auto-provisioned page must start unpublished");
    assert!(layouts[0].draft.root.is_empty(), "auto-provisioned page starts with an empty canvas");
}

#[test]
fn a_new_page_is_not_default_and_the_first_ever_page_is() {
    let (conn, ws, admin) = setup_workspace();
    page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let second = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&admin)).unwrap();
    assert!(!second.is_default);
}

#[test]
fn the_first_page_created_for_an_entity_becomes_the_default() {
    let (conn, ws, admin) = setup_workspace();
    let only = page_layout_service::create_layout(&conn, &ws, &layout_input("Only page"), Some(&admin)).unwrap();
    assert!(only.is_default);
}

#[test]
fn a_non_administrator_cannot_create_a_page() {
    let (conn, ws, admin) = setup_workspace();
    let sam = sales_user(&conn, &ws, &admin);
    let result = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&sam));
    assert!(result.is_err());
}

#[test]
fn publish_unpublish_and_revert_round_trip() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);

    let draft1 = PageDefinition { root: vec![leaf("field", 6)] };
    let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: draft1.clone() };
    let updated = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert!(updated.published.is_none());

    let published = page_layout_service::publish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(published.published, Some(published.draft.clone()));

    let draft2 = PageDefinition { root: vec![leaf("kpi", 3)] };
    let update2 = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: draft2 };
    let edited = page_layout_service::update_layout(&conn, &layout.id, &update2, Some(&admin)).unwrap();
    assert_ne!(edited.draft, edited.published.clone().unwrap());

    let reverted = page_layout_service::revert_layout_draft(&conn, &layout.id, Some(&admin)).unwrap();
    assert_eq!(reverted.draft, reverted.published.clone().unwrap());

    let unpublished = page_layout_service::unpublish_layout(&conn, &layout.id, Some(&admin)).unwrap();
    assert!(unpublished.published.is_none());
}

#[test]
fn make_default_moves_the_flag_and_only_one_page_is_ever_default() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let other = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&admin)).unwrap();
    assert!(!other.is_default);

    let promoted = page_layout_service::make_default(&conn, &other.id, Some(&admin)).unwrap();
    assert!(promoted.is_default);

    let old_default = page_layout_service::get_layout(&conn, &default_layout.id).unwrap();
    assert!(!old_default.is_default);

    let all = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(all.iter().filter(|l| l.is_default).count(), 1);
}

#[test]
fn the_default_page_cannot_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let result = page_layout_service::delete_layout(&conn, &default_layout.id, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn a_non_default_page_can_be_deleted() {
    let (conn, ws, admin) = setup_workspace();
    page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let other = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&admin)).unwrap();
    page_layout_service::delete_layout(&conn, &other.id, Some(&admin)).unwrap();
    let remaining = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    assert_eq!(remaining.len(), 1);
}

#[test]
fn resolve_effective_page_returns_none_when_nothing_is_published() {
    let (conn, ws, _admin) = setup_workspace();
    let effective = page_layout_service::resolve_effective_page(&conn, &ws, "Company", None).unwrap();
    assert!(effective.is_none());
}

#[test]
fn resolve_effective_page_falls_back_to_the_published_default_for_an_unclaimed_role() {
    let (conn, ws, admin) = setup_workspace();
    let default_layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let draft = PageDefinition { root: vec![leaf("record_header", 12)] };
    let update = PageLayoutUpdate { name: default_layout.name.clone(), roles: vec![], draft: draft.clone() };
    page_layout_service::update_layout(&conn, &default_layout.id, &update, Some(&admin)).unwrap();
    page_layout_service::publish_layout(&conn, &default_layout.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = page_layout_service::resolve_effective_page(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(effective.unwrap(), draft);
}

#[test]
fn resolve_effective_page_prefers_a_published_page_matching_the_actors_role() {
    let (conn, ws, admin) = setup_workspace();
    page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_page = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&admin)).unwrap();
    let draft = PageDefinition { root: vec![leaf("kpi", 3)] };
    let update = PageLayoutUpdate { name: sales_page.name.clone(), roles: vec!["Sales".into()], draft: draft.clone() };
    page_layout_service::update_layout(&conn, &sales_page.id, &update, Some(&admin)).unwrap();
    page_layout_service::publish_layout(&conn, &sales_page.id, Some(&admin)).unwrap();

    let sam = sales_user(&conn, &ws, &admin);
    let effective = page_layout_service::resolve_effective_page(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert_eq!(effective.unwrap(), draft);

    // The Administrator (no Sales role) falls back to Default, never
    // published here, so still None.
    let admin_effective = page_layout_service::resolve_effective_page(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert!(admin_effective.is_none());
}

#[test]
fn an_unpublished_role_matching_page_is_never_used_live() {
    let (conn, ws, admin) = setup_workspace();
    page_layout_service::list_layouts(&conn, &ws, "Company").unwrap();
    let sales_page = page_layout_service::create_layout(&conn, &ws, &layout_input("Sales page"), Some(&admin)).unwrap();
    let update = PageLayoutUpdate { name: sales_page.name.clone(), roles: vec!["Sales".into()], draft: PageDefinition { root: vec![leaf("kpi", 3)] } };
    page_layout_service::update_layout(&conn, &sales_page.id, &update, Some(&admin)).unwrap();
    // Deliberately never published.

    let sam = sales_user(&conn, &ws, &admin);
    let effective = page_layout_service::resolve_effective_page(&conn, &ws, "Company", Some(&sam)).unwrap();
    assert!(effective.is_none(), "a draft-only page must never affect the live record view");
}

#[test]
fn an_invalid_entity_type_is_rejected() {
    let (conn, ws, _admin) = setup_workspace();
    let result = page_layout_service::list_layouts(&conn, &ws, "NotARealEntity");
    assert!(result.is_err());
}

// --- Component-type validation ---

#[test]
fn every_component_type_alone_at_the_root_saves_successfully() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    for component_type in COMPONENT_TYPES {
        let draft = PageDefinition { root: vec![leaf(component_type, 12)] };
        let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
        let result = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin));
        assert!(result.is_ok(), "'{component_type}' should be a valid standalone node, got {result:?}");
    }
}

#[test]
fn every_container_type_accepts_a_field_child_and_every_leaf_type_rejects_one() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    for component_type in COMPONENT_TYPES {
        let draft = PageDefinition { root: vec![container(component_type, vec![leaf("field", 6)])] };
        let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
        let result = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin));
        if CONTAINER_COMPONENT_TYPES.contains(component_type) {
            assert!(result.is_ok(), "'{component_type}' is a container and should accept a child, got {result:?}");
        } else {
            assert!(result.is_err(), "'{component_type}' is a leaf and should reject a child");
        }
    }
}

#[test]
fn an_unknown_component_type_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let draft = PageDefinition { root: vec![leaf("not_a_real_component", 12)] };
    let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
    let result = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn an_out_of_range_column_span_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    for bad_span in [0u8, 13u8] {
        let draft = PageDefinition { root: vec![leaf("field", bad_span)] };
        let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft };
        let result = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin));
        assert!(result.is_err(), "column_span {bad_span} should be rejected");
    }
}

#[test]
fn a_deeply_nested_valid_tree_saves_and_round_trips() {
    let (conn, ws, admin) = setup_workspace();
    let layout = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let draft = PageDefinition {
        root: vec![container(
            "section",
            vec![container("columns", vec![leaf("field", 6), container("field_group", vec![leaf("field", 6)])])],
        )],
    };
    let update = PageLayoutUpdate { name: layout.name.clone(), roles: vec![], draft: draft.clone() };
    let saved = page_layout_service::update_layout(&conn, &layout.id, &update, Some(&admin)).unwrap();
    assert_eq!(saved.draft, draft);

    let reloaded = page_layout_service::get_layout(&conn, &layout.id).unwrap();
    assert_eq!(reloaded.draft, draft);
}

// --- Parity gate: Screen Builder 2.0 must leave Screen/App Builder
// (Phase 1-3) completely unaffected - a separate table, a separate
// service, no shared mutable state. ---

#[test]
fn page_layouts_and_screen_layouts_are_completely_independent_per_entity() {
    let (conn, ws, admin) = setup_workspace();

    // Both auto-provision their own unrelated Default the first time
    // either is listed for "Company".
    let screen_default = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    let page_default = page_layout_service::list_layouts(&conn, &ws, "Company").unwrap().remove(0);
    assert_ne!(screen_default.id, page_default.id);

    // Publishing a real draft on the ScreenLayout side...
    let screen_update = ScreenLayoutUpdate {
        name: screen_default.name.clone(),
        roles: vec![],
        draft: LayoutTabs { tabs: vec![LayoutTab { id: "t1".into(), title: "Details".into(), sections: vec![], related: vec![] }] },
    };
    screen_layout_service::update_layout(&conn, &screen_default.id, &screen_update, Some(&admin)).unwrap();
    screen_layout_service::publish_layout(&conn, &screen_default.id, Some(&admin)).unwrap();

    // ...must never surface as an effective PageLayout, and vice versa.
    let effective_page = page_layout_service::resolve_effective_page(&conn, &ws, "Company", None).unwrap();
    assert!(effective_page.is_none(), "publishing a ScreenLayout must not publish a PageLayout");

    let page_update = PageLayoutUpdate { name: page_default.name.clone(), roles: vec![], draft: PageDefinition { root: vec![leaf("record_header", 12)] } };
    page_layout_service::update_layout(&conn, &page_default.id, &page_update, Some(&admin)).unwrap();
    page_layout_service::publish_layout(&conn, &page_default.id, Some(&admin)).unwrap();

    let effective_screen = screen_layout_service::resolve_effective_layout(&conn, &ws, "Company", None).unwrap();
    assert_eq!(
        effective_screen,
        Some(LayoutTabs { tabs: vec![LayoutTab { id: "t1".into(), title: "Details".into(), sections: vec![], related: vec![] }] }),
        "publishing a PageLayout must not change what resolve_effective_layout (ScreenLayout) returns"
    );

    // Deleting every PageLayout for this entity down to its own Default
    // must leave the ScreenLayout row count untouched.
    let screen_count_before = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().len();
    let extra_page = page_layout_service::create_layout(&conn, &ws, &layout_input("Extra page"), Some(&admin)).unwrap();
    page_layout_service::delete_layout(&conn, &extra_page.id, Some(&admin)).unwrap();
    let screen_count_after = screen_layout_service::list_layouts(&conn, &ws, "Company").unwrap().len();
    assert_eq!(screen_count_before, screen_count_after);
}

// Note: `page_layouts.workspace_id` declares the identical `REFERENCES
// workspaces(id) ON DELETE CASCADE` screen_layouts already relies on -
// not re-tested with a direct `DELETE FROM workspaces` here, since a
// fresh workspace has other FK-constrained rows (its admin user, seed
// numbering configs, etc.) that make a raw single-table delete fail for
// reasons having nothing to do with this table; no other test in this
// suite exercises workspace deletion that way either.
