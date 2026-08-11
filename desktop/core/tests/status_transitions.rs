//! Admin Automation & Customization addendum, Phase 2 (spec §2.5): the
//! Status Transition editor - an allow-list of From -> To pairs an
//! Administrator can define per entity type, enforced at each entity's
//! status-changing call site. With zero rules defined, transitions stay
//! fully unrestricted (backward compatible default).

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::status_transition::StatusTransitionInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{company_service, opportunity_service, quote_service, status_transition_service, user_service, workspace_service};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Status Transition Test Co".into(),
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

fn company_input(name: &str, status: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(), status: status.into(), owner_user_id: None, tax_number: None,
        billing_address: None, shipping_address: None, tags: None, notes: None,
    }
}

fn rule(entity_type: &str, from_status: Option<&str>, to_status: &str) -> StatusTransitionInput {
    StatusTransitionInput { entity_type: entity_type.into(), from_status: from_status.map(String::from), to_status: to_status.into() }
}

#[test]
fn no_rules_means_transitions_stay_fully_unrestricted() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    // No status_transitions rows exist for Company at all - any jump works.
    let updated = company_service::update(&conn, &company.id, &company_input("Acme", "Archived"), Some(&admin)).unwrap();
    assert_eq!(updated.status, "Archived");
}

#[test]
fn a_rule_restricts_transitions_to_the_listed_pairs_once_one_exists() {
    let (conn, ws, admin) = setup_workspace();
    status_transition_service::create(&conn, &ws, &rule("Company", Some("Prospect"), "Active Customer"), Some(&admin)).unwrap();

    let allowed = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let updated = company_service::update(&conn, &allowed.id, &company_input("Acme", "Active Customer"), Some(&admin)).unwrap();
    assert_eq!(updated.status, "Active Customer");

    let blocked = company_service::create(&conn, &ws, &company_input("Globex", "Prospect"), Some(&admin)).unwrap();
    let err = company_service::update(&conn, &blocked.id, &company_input("Globex", "Inactive"), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("cannot move from 'Prospect' to 'Inactive'"));
}

#[test]
fn a_wildcard_rule_allows_the_transition_from_any_starting_status() {
    let (conn, ws, admin) = setup_workspace();
    // "-> Archived" from any status, no from_status restriction.
    status_transition_service::create(&conn, &ws, &rule("Company", None, "Archived"), Some(&admin)).unwrap();

    for start_status in ["Prospect", "Active Customer", "Inactive"] {
        let c = company_service::create(&conn, &ws, &company_input("Test", start_status), Some(&admin)).unwrap();
        let updated = company_service::update(&conn, &c.id, &company_input("Test", "Archived"), Some(&admin)).unwrap();
        assert_eq!(updated.status, "Archived");
    }

    // But once a rule set exists, moving to anything NOT listed still fails.
    let c = company_service::create(&conn, &ws, &company_input("Test2", "Prospect"), Some(&admin)).unwrap();
    let err = company_service::update(&conn, &c.id, &company_input("Test2", "Inactive"), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("isn't allowed"));
}

#[test]
fn resaving_the_same_status_is_never_blocked() {
    let (conn, ws, admin) = setup_workspace();
    // A rule set so restrictive it allows nothing - re-saving unchanged
    // still must always succeed (old_status == new_status is a no-op).
    status_transition_service::create(&conn, &ws, &rule("Company", Some("Prospect"), "Active Customer"), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let updated = company_service::update(&conn, &company.id, &company_input("Acme Renamed", "Prospect"), Some(&admin)).unwrap();
    assert_eq!(updated.name, "Acme Renamed");
}

#[test]
fn opportunity_stage_transitions_are_enforced_on_the_stage_field_not_status() {
    let (conn, ws, admin) = setup_workspace();
    // Any status can reach Negotiation, but Won is reachable only from
    // Negotiation - so a fresh Opportunity must pass through Negotiation
    // first rather than jumping straight to Won.
    status_transition_service::create(&conn, &ws, &rule("Opportunity", None, "Negotiation"), Some(&admin)).unwrap();
    status_transition_service::create(&conn, &ws, &rule("Opportunity", Some("Negotiation"), "Won"), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let opp_input = |stage: &str| OpportunityInput {
        company_id: company.id.clone(), primary_contact_id: None, name: "Deal".into(), stage: stage.into(),
        status: if stage == "Won" { "Won" } else { "Open" }.into(), value_cents: 100000, currency_code: "USD".into(),
        probability_bp: 0, expected_close_date: None, owner_user_id: None, lost_reason: None, next_step: None,
    };
    let opp = opportunity_service::create(&conn, &opp_input("New"), Some(&admin)).unwrap();

    // Skipping straight from "New" to "Won" isn't a listed pair.
    let err = opportunity_service::update(&conn, &opp.id, &opp_input("Won"), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("Opportunity cannot move from 'New' to 'Won'"));

    // Going through Negotiation first, then Won, both succeed.
    opportunity_service::update(&conn, &opp.id, &opp_input("Negotiation"), Some(&admin)).unwrap();
    let won = opportunity_service::update(&conn, &opp.id, &opp_input("Won"), Some(&admin)).unwrap();
    assert_eq!(won.stage, "Won");
}

#[test]
fn quote_set_status_is_enforced_the_same_way_as_a_generic_update() {
    let (conn, ws, admin) = setup_workspace();
    status_transition_service::create(&conn, &ws, &rule("Quote", Some("Draft"), "Sent"), Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    let quote = quote_service::create(
        &conn,
        &lanesra_core::models::quote::QuoteInput {
            company_id: company.id.clone(), contact_id: None, opportunity_id: None,
            currency_code: "USD".into(), issue_date: None, expiry_date: None, notes: None, terms: None,
            lines: vec![lanesra_core::models::quote::QuoteLineInput {
                product_id: None, description: "Consulting".into(), quantity_milli: 1000,
                unit_price_cents: 10000, discount_bp: 0, tax_rate_bp: 0,
            }],
        },
        Some(&admin),
    ).unwrap();

    quote_service::set_status(&conn, &quote.quote.id, "Sent", Some(&admin)).unwrap();
    // Draft -> Sent already used; Sent -> Accepted has no rule.
    let err = quote_service::set_status(&conn, &quote.quote.id, "Accepted", Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("cannot move from 'Sent' to 'Accepted'"));
}

#[test]
fn non_administrator_cannot_manage_status_transition_rules() {
    let (conn, ws, admin) = setup_workspace();
    let sales_user = user_service::create(
        &conn, &ws,
        &NewUser { username: "sam".into(), display_name: "Sam".into(), password: "anothersecretpw".into(), roles: vec!["Sales".into()] },
        Some(&admin),
    ).unwrap();
    let err = status_transition_service::create(&conn, &ws, &rule("Company", Some("Prospect"), "Active Customer"), Some(&sales_user.id)).unwrap_err();
    assert!(format!("{err:?}").contains("Only an Administrator"));
}

#[test]
fn an_invalid_to_status_is_rejected_at_definition_time() {
    let (conn, ws, admin) = setup_workspace();
    let err = status_transition_service::create(&conn, &ws, &rule("Company", None, "Not A Real Status"), Some(&admin)).unwrap_err();
    assert!(format!("{err:?}").contains("is not a valid status"));
}

#[test]
fn deleting_a_rule_removes_it_from_enforcement() {
    let (conn, ws, admin) = setup_workspace();
    let created = status_transition_service::create(&conn, &ws, &rule("Company", Some("Prospect"), "Active Customer"), Some(&admin)).unwrap();
    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    // Blocked while the rule exists and requires Prospect->Active Customer only.
    assert!(company_service::update(&conn, &company.id, &company_input("Acme", "Inactive"), Some(&admin)).is_err());

    status_transition_service::delete(&conn, &created.id, Some(&admin)).unwrap();
    // With the last rule gone, the entity type is unrestricted again.
    let updated = company_service::update(&conn, &company.id, &company_input("Acme", "Inactive"), Some(&admin)).unwrap();
    assert_eq!(updated.status, "Inactive");
}

#[test]
fn deactivating_a_rule_stops_it_from_being_enforced_without_deleting_it() {
    let (conn, ws, admin) = setup_workspace();
    let created = status_transition_service::create(&conn, &ws, &rule("Company", Some("Prospect"), "Active Customer"), Some(&admin)).unwrap();
    status_transition_service::set_active(&conn, &created.id, false, Some(&admin)).unwrap();

    let company = company_service::create(&conn, &ws, &company_input("Acme", "Prospect"), Some(&admin)).unwrap();
    // The only rule is now inactive, so Company transitions are unrestricted again.
    let updated = company_service::update(&conn, &company.id, &company_input("Acme", "Inactive"), Some(&admin)).unwrap();
    assert_eq!(updated.status, "Inactive");

    let rules = status_transition_service::list(&conn, &ws, "Company", Some(&admin)).unwrap();
    assert_eq!(rules.len(), 1);
    assert!(!rules[0].is_active);
}
