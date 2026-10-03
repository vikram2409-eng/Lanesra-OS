//! Screen Builder 2.0 (issue #195, 5b): "admins can save a customized page
//! as an Organization Template." Exercises page_template_service in
//! isolation plus an explicit parity test proving it never touches
//! page_layouts' own rows - see page_template_service.rs's own doc
//! comment for why a template has no draft/published/roles lifecycle.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::page_layout::{NodeLayout, PageDefinition, PageLayoutInput, PageLayoutUpdate, PageNode};
use lanesra_core::models::page_template::PageTemplateInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{page_layout_service, page_template_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Template Co".into(),
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

fn leaf(component_type: &str) -> PageNode {
    PageNode {
        id: format!("n-{component_type}"),
        component_type: component_type.into(),
        config: serde_json::json!({}),
        children: vec![],
        layout: NodeLayout { column_span: 6, tablet_column_span: None, mobile_column_span: None, order: 0 },
    }
}

/// Creates a page for "Company" with one `record_header` node on its
/// draft, returning its id.
fn page_with_one_node(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    page_layout_service::list_layouts(conn, ws, "Company").unwrap();
    let layout = page_layout_service::create_layout(conn, ws, &PageLayoutInput { entity_type: "Company".into(), name: "Sales page".into() }, Some(admin)).unwrap();
    let update = PageLayoutUpdate {
        name: layout.name.clone(),
        roles: vec![],
        draft: PageDefinition { root: vec![leaf("record_header")] },
    };
    page_layout_service::update_layout(conn, &layout.id, &update, Some(admin)).unwrap();
    layout.id
}

fn template_input(name: &str) -> PageTemplateInput {
    PageTemplateInput { name: name.into(), description: Some("A saved template".into()) }
}

#[test]
fn creating_a_template_snapshots_the_pages_current_draft() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let template = page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&admin)).unwrap();
    assert_eq!(template.name, "Exec 360");
    assert_eq!(template.entity_type, "Company");
    assert_eq!(template.definition.root.len(), 1);
    assert_eq!(template.definition.root[0].component_type, "record_header");
}

#[test]
fn editing_the_page_after_saving_a_template_does_not_change_the_template() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let template = page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&admin)).unwrap();

    let page = page_layout_service::get_layout(&conn, &page_id).unwrap();
    let update = PageLayoutUpdate { name: page.name, roles: vec![], draft: PageDefinition { root: vec![leaf("owner"), leaf("note")] } };
    page_layout_service::update_layout(&conn, &page_id, &update, Some(&admin)).unwrap();

    let templates = page_template_service::list_templates(&conn, &ws, "Company").unwrap();
    let still = templates.iter().find(|t| t.id == template.id).unwrap();
    assert_eq!(still.definition.root.len(), 1, "a template is an immutable snapshot, not a live reference to the page");
    assert_eq!(still.definition.root[0].component_type, "record_header");
}

#[test]
fn a_non_administrator_cannot_create_a_template() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let sam = sales_user(&conn, &ws, &admin);
    let result = page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&sam));
    assert!(result.is_err());
}

#[test]
fn a_template_name_is_required() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let result = page_template_service::create_template_from_page(&conn, &ws, &page_id, &PageTemplateInput { name: "   ".into(), description: None }, Some(&admin));
    assert!(result.is_err());
}

// A real second workspace can't be set up to test this against - this
// codebase's desktop/SQLite model is one workspace per connection/db file
// (`first_run_setup` itself refuses a second one; see the Postgres-per-
// tenant roadmap item for why multi-tenancy-in-one-database is deferred,
// not supported here) - so this exercises the service's own defensive
// `page.workspace_id != workspace_id` guard directly with a workspace id
// that simply isn't the real one, which is the only way this branch is
// actually reachable in this architecture.
#[test]
fn a_page_cannot_be_templated_under_the_wrong_workspace_id() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let result = page_template_service::create_template_from_page(&conn, "not-the-real-workspace-id", &page_id, &template_input("Stolen"), Some(&admin));
    assert!(result.is_err());
}

#[test]
fn listing_templates_is_scoped_to_entity_type() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&admin)).unwrap();

    let company_templates = page_template_service::list_templates(&conn, &ws, "Company").unwrap();
    assert_eq!(company_templates.len(), 1);
    let contact_templates = page_template_service::list_templates(&conn, &ws, "Contact").unwrap();
    assert_eq!(contact_templates.len(), 0);
}

#[test]
fn deleting_a_template_removes_it() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let template = page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&admin)).unwrap();
    page_template_service::delete_template(&conn, &template.id, &ws, Some(&admin)).unwrap();
    let remaining = page_template_service::list_templates(&conn, &ws, "Company").unwrap();
    assert!(remaining.is_empty());
}

/// Parity gate: saving/deleting templates must never touch the page's own
/// row - the exact page_layouts row that was templated is still there,
/// unchanged, with its own independent draft/published lifecycle intact.
#[test]
fn templates_and_page_layouts_are_completely_independent() {
    let (conn, ws, admin) = setup_workspace();
    let page_id = page_with_one_node(&conn, &ws, &admin);
    let before = page_layout_service::get_layout(&conn, &page_id).unwrap();

    let template = page_template_service::create_template_from_page(&conn, &ws, &page_id, &template_input("Exec 360"), Some(&admin)).unwrap();
    page_template_service::delete_template(&conn, &template.id, &ws, Some(&admin)).unwrap();

    let after = page_layout_service::get_layout(&conn, &page_id).unwrap();
    assert_eq!(before.draft, after.draft);
    assert_eq!(before.name, after.name);
    assert_eq!(before.is_default, after.is_default);
}
