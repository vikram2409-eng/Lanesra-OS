//! Global search (roadmap "Global search & list-view filtering"): exercises
//! every branch of `search_service::global_search` - a natural-field match
//! per core entity, an active custom object match, an `is_searchable`
//! custom field match (and that a non-searchable field is correctly
//! ignored), archived-record exclusion, the short-query cutoff, and the
//! MAX_RESULTS cap.

use std::collections::HashMap;

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::company::CompanyInput;
use lanesra_core::models::contact::ContactInput;
use lanesra_core::models::contract::ContractInput;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::opportunity::OpportunityInput;
use lanesra_core::models::product::ProductInput;
use lanesra_core::models::quote::{QuoteInput, QuoteLineInput};
use lanesra_core::models::task::TaskInput;
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::services::{
    company_service, contact_service, contract_service, custom_field_service, custom_object_service,
    custom_record_service, invoice_service, opportunity_service, order_service, product_service,
    quote_service, search_service, task_service, workspace_service,
};

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Search Test Co".into(),
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

fn company_input(name: &str) -> CompanyInput {
    CompanyInput {
        name: name.into(),
        status: "Prospect".into(),
        owner_user_id: None,
        tax_number: None,
        billing_address: None,
        shipping_address: None,
        tags: None,
        notes: None,
        ..Default::default()
    }
}

fn contact_input(company_id: &str, first_name: &str, last_name: &str) -> ContactInput {
    ContactInput {
        company_id: company_id.into(),
        first_name: first_name.into(),
        last_name: last_name.into(),
        job_title: None,
        email: Some("zaphod.contact@example.com".into()),
        phone: None,
        mobile: None,
        is_primary: true,
        status: "Active".into(),
        tags: None,
        notes: None,
        ..Default::default()
    }
}

fn opportunity_input(company_id: &str, name: &str) -> OpportunityInput {
    OpportunityInput {
        company_id: company_id.into(),
        primary_contact_id: None,
        name: name.into(),
        stage: "New".into(),
        status: "Open".into(),
        value_cents: 10000,
        currency_code: "USD".into(),
        probability_bp: 1000,
        expected_close_date: None,
        owner_user_id: None,
        lost_reason: None,
        next_step: None,
    }
}

fn product_input(name: &str) -> ProductInput {
    ProductInput {
        sku: Some("ZPH-1000".into()),
        r#type: "Product".into(),
        name: name.into(),
        category: None,
        description: None,
        unit_price_cents: 1000,
        cost_cents: 500,
        tax_rate_bp: 0,
        default_quantity_milli: 1000,
        is_active: true,
    }
}

fn contract_input(company_id: &str, title: &str) -> ContractInput {
    ContractInput {
        company_id: company_id.into(),
        contact_id: None,
        source_quote_id: None,
        title: title.into(),
        r#type: Some("MSA".into()),
        value_cents: 100000,
        currency_code: "USD".into(),
        owner_user_id: None,
        start_date: None,
        end_date: None,
        renewal_date: None,
        notice_period_days: Some(30),
        status: "Active".into(),
        notes: None,
    }
}

fn task_input(title: &str) -> TaskInput {
    TaskInput {
        title: title.into(),
        description: None,
        owner_user_id: None,
        priority: "Normal".into(),
        status: "Not Started".into(),
        due_date: None,
        reminder_at: None,
        related_type: None,
        related_id: None,
    }
}

fn vendor_object_input() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput {
        singular_label: "Vendor".into(),
        plural_label: "Vendors".into(),
        icon: "🏭".into(),
        prefix: "VEN".into(),
        digits: 4,
    }
}

fn searchable_text_field(entity_type: &str, label: &str, is_searchable: bool) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: entity_type.into(),
        label: label.into(),
        field_type: "text".into(),
        options: vec![],
        required: false,
        show_in_list: false,
        sort_order: 0,
        min_value: None,
        max_value: None,
        max_length: None,
        regex_pattern: None,
        is_searchable,
        is_filterable: false,
        is_reportable: false,
        default_value: None,
        is_unique: false,
        help_text: None,
        placeholder: None,
        is_hidden_by_default: false,
    }
}

#[test]
fn a_short_query_returns_no_results() {
    let (conn, ws, admin) = setup_workspace();
    company_service::create(&conn, &ws, &company_input("Zaphod Holdings"), Some(&admin)).unwrap();

    assert!(search_service::global_search(&conn, &ws, "").unwrap().is_empty());
    assert!(search_service::global_search(&conn, &ws, "Z").unwrap().is_empty());
    assert!(search_service::global_search(&conn, &ws, " ").unwrap().is_empty());
}

#[test]
fn matches_a_company_by_name() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Zaphod Holdings"), Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].entity_type, "Company");
    assert_eq!(results[0].entity_id, company.id);
    assert_eq!(results[0].title, "Zaphod Holdings");
}

#[test]
fn matches_a_contact_by_name_and_shows_email_as_subtitle() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Beeblebrox Inc"), Some(&admin)).unwrap();
    let contact = contact_service::create(&conn, &contact_input(&company.id, "Zaphod", "Beeblebrox"), Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    let hit = results.iter().find(|r| r.entity_type == "Contact").expect("contact match");
    assert_eq!(hit.entity_id, contact.id);
    assert_eq!(hit.title, "Zaphod Beeblebrox");
    assert_eq!(hit.subtitle.as_deref(), Some("zaphod.contact@example.com"));
}

#[test]
fn matches_an_opportunity_a_product_a_contract_and_a_task_by_their_natural_fields() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Heart of Gold Ltd"), Some(&admin)).unwrap();

    let opportunity = opportunity_service::create(&conn, &opportunity_input(&company.id, "Zaphod Expansion Deal"), Some(&admin)).unwrap();
    let product = product_service::create(&conn, &ws, &product_input("Zaphod Widget"), Some(&admin)).unwrap();
    let contract = contract_service::create(&conn, &contract_input(&company.id, "Zaphod Services Agreement"), Some(&admin)).unwrap();
    let task = task_service::create(&conn, &ws, &task_input("Follow up with Zaphod"), Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    let by_type = |t: &str| results.iter().find(|r| r.entity_type == t);

    assert_eq!(by_type("Opportunity").unwrap().entity_id, opportunity.id);
    assert_eq!(by_type("Product").unwrap().entity_id, product.id);
    assert_eq!(by_type("Contract").unwrap().entity_id, contract.id);
    assert_eq!(by_type("Task").unwrap().entity_id, task.id);
}

#[test]
fn matches_a_quote_an_order_and_an_invoice_by_number() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Megadodo Publications"), Some(&admin)).unwrap();

    let quote = quote_service::create(
        &conn,
        &QuoteInput {
            company_id: company.id.clone(),
            contact_id: None,
            opportunity_id: None,
            currency_code: "USD".into(),
            issue_date: None,
            expiry_date: None,
            notes: None,
            terms: None,
            lines: vec![QuoteLineInput {
                product_id: None,
                description: "Consulting".into(),
                quantity_milli: 1000,
                unit_price_cents: 10000,
                discount_bp: 0,
                tax_rate_bp: 0,
            }],
        },
        Some(&admin),
    )
    .unwrap();
    let quote_number = quote.quote.quote_number.clone();

    quote_service::set_status(&conn, &quote.quote.id, "Sent", Some(&admin)).unwrap();
    quote_service::set_status(&conn, &quote.quote.id, "Accepted", Some(&admin)).unwrap();
    let order = quote_service::convert_to_order(&conn, &quote.quote.id, Some(&admin)).unwrap();
    let order_number = order.order.order_number.clone();

    order_service::set_status(&conn, &order.order.id, "Confirmed", Some(&admin)).unwrap();
    let invoice = order_service::convert_to_invoice(&conn, &order.order.id, Some(&admin)).unwrap();
    let invoice_number = invoice.invoice.invoice_number.clone();
    invoice_service::issue(&conn, &invoice.invoice.id, Some(&admin)).unwrap();

    let quote_results = search_service::global_search(&conn, &ws, &quote_number).unwrap();
    assert!(quote_results.iter().any(|r| r.entity_type == "Quote" && r.entity_id == quote.quote.id));

    let order_results = search_service::global_search(&conn, &ws, &order_number).unwrap();
    assert!(order_results.iter().any(|r| r.entity_type == "Order" && r.entity_id == order.order.id));

    let invoice_results = search_service::global_search(&conn, &ws, &invoice_number).unwrap();
    assert!(invoice_results.iter().any(|r| r.entity_type == "Invoice" && r.entity_id == invoice.invoice.id));
}

#[test]
fn matches_an_active_custom_objects_record() {
    let (conn, ws, admin) = setup_workspace();
    let vendor_def = custom_object_service::create(&conn, &ws, &vendor_object_input(), Some(&admin)).unwrap();
    let vendor = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput {
            object_key: vendor_def.key.clone(),
            primary_name: "Zaphod Supply Co".into(),
            status: "Active".into(),
            owner_user_id: None,
            notes: None,
        },
        Some(&admin),
    )
    .unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    let hit = results.iter().find(|r| r.entity_type == vendor_def.key).expect("vendor record match");
    assert_eq!(hit.entity_id, vendor.id);
    assert_eq!(hit.title, "Zaphod Supply Co");
}

#[test]
fn matches_a_searchable_custom_field_value_and_resolves_the_owning_records_display_name() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Infinite Improbability Inc"), Some(&admin)).unwrap();
    let field = custom_field_service::create_definition(&conn, &ws, &searchable_text_field("Company", "Nickname", true), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(field.key.clone(), "Zaphod's Favorite".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    let hit = results
        .iter()
        .find(|r| r.entity_type == "Company" && r.entity_id == company.id && r.subtitle.is_some())
        .expect("custom field match");
    assert_eq!(hit.title, "Infinite Improbability Inc");
    assert_eq!(hit.subtitle.as_deref(), Some("Nickname: Zaphod's Favorite"));
}

#[test]
fn a_non_searchable_custom_field_value_is_never_matched() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Golgafrincham Ark Corp"), Some(&admin)).unwrap();
    let field = custom_field_service::create_definition(&conn, &ws, &searchable_text_field("Company", "Internal Code", false), Some(&admin)).unwrap();

    let mut values = HashMap::new();
    values.insert(field.key.clone(), "ZAPHODCODE".into());
    custom_field_service::set_entity_values(&conn, "Company", &company.id, &values, Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "zaphodcode").unwrap();
    assert!(results.is_empty(), "a non-searchable custom field must never be matched");
}

#[test]
fn archived_records_are_excluded_from_results() {
    let (conn, ws, admin) = setup_workspace();
    let company = company_service::create(&conn, &ws, &company_input("Vogon Constructor Fleet"), Some(&admin)).unwrap();
    company_service::archive(&conn, &company.id, Some(&admin)).unwrap();

    let results = search_service::global_search(&conn, &ws, "vogon").unwrap();
    assert!(results.is_empty(), "an archived record must not appear in search results");
}

#[test]
fn results_are_capped_at_max_results() {
    let (conn, ws, admin) = setup_workspace();
    for i in 0..40 {
        company_service::create(&conn, &ws, &company_input(&format!("Zaphod Branch {i:02}")), Some(&admin)).unwrap();
    }

    let results = search_service::global_search(&conn, &ws, "zaphod").unwrap();
    assert_eq!(results.len(), 25);
}
