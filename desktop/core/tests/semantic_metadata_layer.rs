//! Next-Gen program, Domain A (Intelligence Foundation), FND-02: the
//! Semantic Metadata Layer. Covers FND-02's own acceptance criterion
//! directly - a glossary term or metric can be traced to the exact
//! metadata and sources it describes via the System Graph (FND-01) -
//! plus validation, version history, and the
//! soft-deactivate-keeps/hard-delete-cascades distinction established
//! for every other System Graph node type.

use lanesra_core::domain::AppError;
use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::semantic::{BusinessGlossaryTermInput, MetricDefinitionInput, SemanticMappingInput};
use lanesra_core::services::{custom_field_service, custom_object_service, glossary_service, metric_service, semantic_mapping_service, system_graph_service};

fn setup_workspace(business_name: &str) -> (rusqlite::Connection, String, String) {
    let conn = lanesra_core::db::open_in_memory_db().unwrap();
    let setup = lanesra_core::models::workspace::WorkspaceSetup {
        business_name: business_name.into(),
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
    let (workspace, admin) = lanesra_core::services::workspace_service::first_run_setup(&conn, &setup).unwrap();
    (conn, workspace.id, admin.id)
}

fn vendor_input() -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: "Vendor".into(), plural_label: "Vendors".into(), icon: "🏭".into(), prefix: "VEN".into(), digits: 4 }
}

fn tier_field_input(entity_type: &str) -> CustomFieldDefinitionInput {
    CustomFieldDefinitionInput {
        entity_type: entity_type.into(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
        show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
        is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: false,
        is_hidden_by_default: false,
    }
}

fn glossary_input(name: &str) -> BusinessGlossaryTermInput {
    BusinessGlossaryTermInput { name: name.into(), definition: "A test term.".into(), owner_user_id: None, synonyms: vec![], data_classification: "standard".into() }
}

fn metric_input(name: &str, entity_type: &str, field_key: Option<String>, glossary_term_id: Option<String>) -> MetricDefinitionInput {
    MetricDefinitionInput {
        name: name.into(), description: None, source_entity_type: entity_type.into(), source_field_key: field_key,
        aggregation: "sum".into(), grain: None, filters_json: "{}".into(), time_logic: None, owner_user_id: None,
        glossary_term_id, effective_start_date: None, effective_end_date: None,
    }
}

#[test]
fn mapping_a_term_to_a_field_syncs_a_derives_from_edge_and_removing_it_removes_the_edge() {
    let (conn, ws, admin) = setup_workspace("Glossary Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();
    let field = custom_field_service::create_definition(&conn, &ws, &tier_field_input(&vendor.key), admin).unwrap();
    let term = glossary_service::create(&conn, &ws, &glossary_input("Vendor Tier"), admin).unwrap();

    // Fresh term has no edges yet - nothing maps to it.
    let lineage = system_graph_service::get_lineage(&conn, &ws, "business_glossary_term", &term.id).unwrap();
    assert!(lineage.is_empty());

    let mapping = semantic_mapping_service::create(
        &conn, &ws,
        &SemanticMappingInput { entity_type: vendor.key.clone(), field_key: Some(field.key.clone()), glossary_term_id: Some(term.id.clone()), semantic_role: None },
        admin,
    )
    .unwrap();

    let lineage = system_graph_service::get_lineage(&conn, &ws, "business_glossary_term", &term.id).unwrap();
    let lineage_ids: Vec<&str> = lineage.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(lineage_ids.contains(&field.id.as_str()), "mapping to a real custom field must resolve to the field's own node, not just the object");

    semantic_mapping_service::delete(&conn, &mapping.id, &ws, admin).unwrap();
    let lineage = system_graph_service::get_lineage(&conn, &ws, "business_glossary_term", &term.id).unwrap();
    assert!(lineage.is_empty(), "removing the only mapping must re-sync the term's edges away, not leave a stale one");
}

#[test]
fn a_metrics_lineage_traces_to_its_source_object_field_and_glossary_term() {
    let (conn, ws, admin) = setup_workspace("Metrics Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();
    let field = custom_field_service::create_definition(&conn, &ws, &tier_field_input(&vendor.key), admin).unwrap();
    let term = glossary_service::create(&conn, &ws, &glossary_input("Spend"), admin).unwrap();

    let metric = metric_service::create(&conn, &ws, &metric_input("Total Spend", &vendor.key, Some(field.key.clone()), Some(term.id.clone())), admin).unwrap();

    let lineage = system_graph_service::get_lineage(&conn, &ws, "metric_definition", &metric.id).unwrap();
    let lineage_ids: Vec<&str> = lineage.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(lineage_ids.contains(&vendor.key.as_str()), "a metric must trace to its source object");
    assert!(lineage_ids.contains(&field.id.as_str()), "a metric must trace to its source field");
    assert!(lineage_ids.contains(&term.id.as_str()), "a metric must trace to the glossary term it derives from");

    // And in the other direction: the source object's impact includes the metric.
    let impact = system_graph_service::get_impact(&conn, &ws, "custom_object", &vendor.key).unwrap();
    let impact_ids: Vec<&str> = impact.iter().map(|h| h.node.component_id.as_str()).collect();
    assert!(impact_ids.contains(&metric.id.as_str()), "the source object's impact must include metrics that depend on it");
}

#[test]
fn invalid_aggregation_role_entity_type_and_an_empty_mapping_are_all_rejected() {
    let (conn, ws, admin) = setup_workspace("Validation Co");
    let admin = Some(admin.as_str());
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();
    let term = glossary_service::create(&conn, &ws, &glossary_input("Some Term"), admin).unwrap();

    let bad_aggregation = metric_input("Bad Metric", &vendor.key, None, None);
    let mut bad_aggregation = bad_aggregation;
    bad_aggregation.aggregation = "median".into();
    assert!(matches!(metric_service::create(&conn, &ws, &bad_aggregation, admin), Err(AppError::Validation(_))), "an unknown aggregation must be rejected");

    let bad_entity = metric_input("Bad Metric 2", "NotARealObject", None, None);
    assert!(matches!(metric_service::create(&conn, &ws, &bad_entity, admin), Err(AppError::Validation(_))), "an unknown source entity type must be rejected");

    let bad_role = SemanticMappingInput { entity_type: vendor.key.clone(), field_key: None, glossary_term_id: None, semantic_role: Some("not_a_real_role".into()) };
    assert!(matches!(semantic_mapping_service::create(&conn, &ws, &bad_role, admin), Err(AppError::Validation(_))), "an unknown semantic role must be rejected");

    let empty_mapping = SemanticMappingInput { entity_type: vendor.key.clone(), field_key: None, glossary_term_id: None, semantic_role: None };
    assert!(matches!(semantic_mapping_service::create(&conn, &ws, &empty_mapping, admin), Err(AppError::Validation(_))), "a mapping with neither a glossary term nor a semantic role must be rejected");

    // Sanity: a well-formed mapping referencing the term created above still succeeds.
    let ok = SemanticMappingInput { entity_type: vendor.key, field_key: None, glossary_term_id: Some(term.id), semantic_role: None };
    assert!(semantic_mapping_service::create(&conn, &ws, &ok, admin).is_ok());
}

#[test]
fn updating_a_metric_appends_a_capped_version_history_row() {
    let (conn, ws, admin) = setup_workspace("Versioning Co");
    let admin = Some(admin.as_str());
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();

    let metric = metric_service::create(&conn, &ws, &metric_input("Order Count", &vendor.key, None, None), admin).unwrap();
    assert!(metric_service::list_versions(&conn, &metric.id, admin).unwrap().is_empty(), "no version is saved until the first update");

    let mut update = metric_input("Order Count", &vendor.key, None, None);
    update.aggregation = "count".into();
    update.grain = Some("daily".into());
    let updated = metric_service::update(&conn, &metric.id, &ws, &update, admin).unwrap();
    assert_eq!(updated.grain.as_deref(), Some("daily"));

    let versions = metric_service::list_versions(&conn, &metric.id, admin).unwrap();
    assert_eq!(versions.len(), 1);
    assert_eq!(versions[0].snapshot.aggregation, "sum", "the version row must capture the metric as it stood *before* this update");

    // 25 further updates (26 total) must prune history down to the 20-row cap.
    for i in 0..25 {
        let mut next = metric_input("Order Count", &vendor.key, None, None);
        next.grain = Some(format!("pass-{i}"));
        metric_service::update(&conn, &metric.id, &ws, &next, admin).unwrap();
    }
    let versions = metric_service::list_versions(&conn, &metric.id, admin).unwrap();
    assert_eq!(versions.len(), 20, "version history must be capped, not grow unbounded");
}

#[test]
fn deactivating_a_glossary_term_keeps_its_node_while_deleting_a_metric_removes_and_cascades_it() {
    let (conn, ws, admin) = setup_workspace("Lifecycle Co");
    let admin = Some(admin.as_str());
    let vendor = custom_object_service::create(&conn, &ws, &vendor_input(), admin).unwrap();

    let term = glossary_service::create(&conn, &ws, &glossary_input("Lifecycle Term"), admin).unwrap();
    glossary_service::deactivate(&conn, &term.id, &ws, admin).unwrap();
    let node = system_graph_service::get_node(&conn, &ws, "business_glossary_term", &term.id).unwrap();
    assert!(node.is_some(), "a soft-deactivated glossary term's node must remain in the graph");

    let metric = metric_service::create(&conn, &ws, &metric_input("Lifecycle Metric", &vendor.key, None, Some(term.id.clone())), admin).unwrap();
    assert!(system_graph_service::get_node(&conn, &ws, "metric_definition", &metric.id).unwrap().is_some());

    let impact_before = system_graph_service::get_impact(&conn, &ws, "custom_object", &vendor.key).unwrap();
    assert!(impact_before.iter().any(|h| h.node.component_id == metric.id));

    metric_service::delete(&conn, &metric.id, &ws, admin).unwrap();
    assert!(system_graph_service::get_node(&conn, &ws, "metric_definition", &metric.id).unwrap().is_none(), "a hard delete must remove the metric's node");
    let impact_after = system_graph_service::get_impact(&conn, &ws, "custom_object", &vendor.key).unwrap();
    assert!(!impact_after.iter().any(|h| h.node.component_id == metric.id), "the deleted metric's own edges must be cascade-removed with it");
}
