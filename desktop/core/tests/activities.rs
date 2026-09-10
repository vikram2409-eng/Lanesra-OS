//! AI & Agentic Layer, Phase 3: the Unified Activity Timeline's generic
//! log - `activity_service::log_activity`/`list_for_entity` against
//! Company, Contact and Opportunity. Mirrors `audit_trail.rs`'s own
//! setup/style.

use lanesra_core::models::activity::ActivityInput;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{activity_service, company_service, contact_service, opportunity_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Activities Test Co".into(),
        legal_name: None,
        currency_code: "USD".into(),
        locale: "en-US".into(),
        timezone: "UTC".into(),
        default_tax_rate_bp: 0,
        admin_username: "admin".into(),
        admin_display_name: "Admin".into(),
        admin_password: "supersecretpassword".into(),
        load_sample_data: false,
    };
    let (workspace, admin) = workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn make_company(conn: &rusqlite::Connection, ws: &str, admin: &str) -> String {
    company_service::create(
        conn, ws,
        &CompanyInput { name: "Acme Corp".into(), status: "Prospect".into(), owner_user_id: None, ..Default::default() },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_contact(conn: &rusqlite::Connection, company_id: &str, admin: &str) -> String {
    contact_service::create(
        conn,
        &ContactInput { company_id: company_id.into(), first_name: "Jane".into(), last_name: "Doe".into(), status: "Active".into(), ..Default::default() },
        Some(admin),
    )
    .unwrap()
    .id
}

fn make_opportunity(conn: &rusqlite::Connection, company_id: &str, admin: &str) -> String {
    opportunity_service::create(
        conn,
        &OpportunityInput {
            company_id: company_id.into(), primary_contact_id: None, name: "Big Deal".into(), stage: "Qualified".into(),
            status: "Open".into(), value_cents: 100_000, currency_code: "USD".into(), probability_bp: 5000,
            expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None,
        },
        Some(admin),
    )
    .unwrap()
    .id
}

fn email_input(entity_type: &str, entity_id: &str) -> ActivityInput {
    ActivityInput {
        entity_type: entity_type.into(),
        entity_id: entity_id.into(),
        channel: "email".into(),
        direction: Some("inbound".into()),
        subject: Some("Following up".into()),
        body: "Circling back on our last conversation.".into(),
        participants: Some("jane@acme.com".into()),
        occurred_at: "2026-01-01T10:00:00Z".into(),
    }
}

#[test]
fn logging_and_listing_round_trips_for_each_supported_entity_type() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let contact_id = make_contact(&conn, &company_id, &admin);
    let opportunity_id = make_opportunity(&conn, &company_id, &admin);

    for (entity_type, entity_id) in [("Company", company_id.as_str()), ("Contact", contact_id.as_str()), ("Opportunity", opportunity_id.as_str())] {
        let logged = activity_service::log_activity(&conn, &email_input(entity_type, entity_id), Some(&admin)).unwrap();
        assert_eq!(logged.entity_type, entity_type);
        assert_eq!(logged.source, "manual");
        assert_eq!(logged.created_by.as_deref(), Some(admin.as_str()));

        let history = activity_service::list_for_entity(&conn, entity_type, entity_id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, logged.id);
        assert_eq!(history[0].subject.as_deref(), Some("Following up"));
    }
}

#[test]
fn an_entity_with_no_logged_activity_returns_an_empty_list() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let history = activity_service::list_for_entity(&conn, "Company", &company_id).unwrap();
    assert!(history.is_empty());
}

#[test]
fn an_unsupported_entity_type_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let result = activity_service::log_activity(&conn, &email_input("Product", &company_id), Some(&admin));
    assert!(result.is_err());
}

#[test]
fn an_unknown_channel_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let mut input = email_input("Company", &company_id);
    input.channel = "carrier_pigeon".into();
    let result = activity_service::log_activity(&conn, &input, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn an_unknown_direction_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let mut input = email_input("Company", &company_id);
    input.direction = Some("sideways".into());
    let result = activity_service::log_activity(&conn, &input, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn an_empty_body_is_rejected() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let mut input = email_input("Company", &company_id);
    input.body = "   ".into();
    let result = activity_service::log_activity(&conn, &input, Some(&admin));
    assert!(result.is_err());
}

#[test]
fn logging_against_a_nonexistent_record_is_not_found() {
    let (conn, _ws, admin) = setup_workspace();
    let result = activity_service::log_activity(&conn, &email_input("Company", "does-not-exist"), Some(&admin));
    assert!(result.is_err());
}

#[test]
fn a_call_activity_needs_no_direction() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);
    let input = ActivityInput {
        entity_type: "Company".into(), entity_id: company_id.clone(), channel: "call".into(), direction: None,
        subject: Some("Discovery call".into()), body: "Discussed pricing and timeline.".into(),
        participants: Some("Jane Doe".into()), occurred_at: "2026-01-02T15:00:00Z".into(),
    };
    let logged = activity_service::log_activity(&conn, &input, Some(&admin)).unwrap();
    assert_eq!(logged.channel, "call");
    assert!(logged.direction.is_none());
}

#[test]
fn most_recent_occurred_at_is_listed_first() {
    let (conn, ws, admin) = setup_workspace();
    let company_id = make_company(&conn, &ws, &admin);

    let mut earlier = email_input("Company", &company_id);
    earlier.occurred_at = "2026-01-01T09:00:00Z".into();
    earlier.subject = Some("Earlier".into());
    activity_service::log_activity(&conn, &earlier, Some(&admin)).unwrap();

    let mut later = email_input("Company", &company_id);
    later.occurred_at = "2026-01-05T09:00:00Z".into();
    later.subject = Some("Later".into());
    activity_service::log_activity(&conn, &later, Some(&admin)).unwrap();

    let history = activity_service::list_for_entity(&conn, "Company", &company_id).unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].subject.as_deref(), Some("Later"));
    assert_eq!(history[1].subject.as_deref(), Some("Earlier"));
}
