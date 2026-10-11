//! Next-Gen program, Ontology Layer epic (issues #340-#345): the unified
//! Object Type registry (issue #341) and Link Type exposure (issue #342).
//! A read model over data that already exists - no migration, no new
//! node type - so these tests exercise the merge/orientation logic
//! `ontology_service` adds on top of `entity_registry`,
//! `custom_object_service`, `relationship_service` and the Business
//! Glossary (FND-02), not a new store.

use lanesra_core::models::custom_field::CustomFieldDefinitionInput;
use lanesra_core::models::custom_object::CustomObjectDefinitionInput;
use lanesra_core::models::relationship::RelationshipDefinitionInput;
use lanesra_core::models::semantic::{BusinessGlossaryTermInput, SemanticMappingInput};
use lanesra_core::services::{custom_field_service, custom_object_service, entity_registry, glossary_service, ontology_service, relationship_service, semantic_mapping_service};

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

fn custom_object_input(singular: &str, plural: &str) -> CustomObjectDefinitionInput {
    CustomObjectDefinitionInput { singular_label: singular.into(), plural_label: plural.into(), icon: "🏭".into(), prefix: "COB".into(), digits: 4 }
}

fn relationship_input(source: &str, target: &str, polymorphic: bool) -> RelationshipDefinitionInput {
    RelationshipDefinitionInput {
        source_entity_type: source.into(),
        target_entity_type: if polymorphic { "".into() } else { target.into() },
        target_is_polymorphic: polymorphic,
        relationship_type: "many_to_one".into(),
        forward_label: "Owning Vendor".into(),
        reverse_label: "Managed Contracts".into(),
        is_required: false,
        show_related_list: true,
        delete_behavior: "restrict".into(),
        sort_order: 0,
    }
}

#[test]
fn list_object_types_includes_every_builtin_and_only_active_custom_objects() {
    let (conn, ws, admin) = setup_workspace("Ontology Co");
    let admin = Some(admin.as_str());

    let active = custom_object_service::create(&conn, &ws, &custom_object_input("Vendor", "Vendors"), admin).unwrap();
    let inactive = custom_object_service::create(&conn, &ws, &custom_object_input("Archive Candidate", "Archive Candidates"), admin).unwrap();
    custom_object_service::deactivate(&conn, &inactive.id, admin).unwrap();

    let types = ontology_service::list_object_types(&conn, &ws).unwrap();
    let keys: Vec<&str> = types.iter().map(|t| t.key.as_str()).collect();

    for builtin in entity_registry::CORE_ENTITY_TYPES {
        assert!(keys.contains(builtin), "builtin object type '{builtin}' must always be listed");
    }
    assert!(keys.contains(&active.key.as_str()), "an active Custom Object must be listed");
    assert!(!keys.contains(&inactive.key.as_str()), "a deactivated Custom Object must not be listed");

    let company = types.iter().find(|t| t.key == "Company").unwrap();
    assert!(!company.is_custom);
    let vendor = types.iter().find(|t| t.key == active.key).unwrap();
    assert!(vendor.is_custom);
    assert_eq!(vendor.label_singular, "Vendor");
    assert_eq!(vendor.label_plural, "Vendors");
}

#[test]
fn description_resolves_only_from_a_whole_object_glossary_mapping() {
    let (conn, ws, admin) = setup_workspace("Ontology Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &custom_object_input("Vendor", "Vendors"), admin).unwrap();
    let field = custom_field_service::create_definition(
        &conn, &ws,
        &CustomFieldDefinitionInput {
            entity_type: vendor.key.clone(), label: "Tier".into(), field_type: "text".into(), options: vec![], required: false,
            show_in_list: true, sort_order: 0, min_value: None, max_value: None, max_length: None, regex_pattern: None,
            is_unique: false, default_value: None, help_text: None, placeholder: None, is_searchable: false, is_filterable: false, is_reportable: false,
            is_hidden_by_default: false,
        },
        admin,
    )
    .unwrap();

    // No mapping yet - no description, never a fabricated one.
    let detail = ontology_service::get_object_type_detail(&conn, &ws, &vendor.key).unwrap();
    assert!(detail.object_type.description.is_none());

    let term = glossary_service::create(
        &conn, &ws,
        &BusinessGlossaryTermInput { name: "Vendor".into(), definition: "A company this workspace buys goods or services from.".into(), owner_user_id: None, synonyms: vec![], data_classification: "standard".into() },
        admin,
    )
    .unwrap();

    // A field-level mapping (field_key set) must not satisfy the
    // whole-object description - only a mapping with field_key = None does.
    semantic_mapping_service::create(
        &conn, &ws,
        &SemanticMappingInput { entity_type: vendor.key.clone(), field_key: Some(field.key.clone()), glossary_term_id: Some(term.id.clone()), semantic_role: None },
        admin,
    )
    .unwrap();
    let detail = ontology_service::get_object_type_detail(&conn, &ws, &vendor.key).unwrap();
    assert!(detail.object_type.description.is_none(), "a field-level mapping must not count as the object's own description");

    semantic_mapping_service::create(
        &conn, &ws,
        &SemanticMappingInput { entity_type: vendor.key.clone(), field_key: None, glossary_term_id: Some(term.id.clone()), semantic_role: None },
        admin,
    )
    .unwrap();
    let detail = ontology_service::get_object_type_detail(&conn, &ws, &vendor.key).unwrap();
    assert_eq!(detail.object_type.description.as_deref(), Some("A company this workspace buys goods or services from."));
}

#[test]
fn link_types_are_oriented_per_side_and_a_polymorphic_target_has_no_fixed_type() {
    let (conn, ws, admin) = setup_workspace("Ontology Co");
    let admin = Some(admin.as_str());

    let vendor = custom_object_service::create(&conn, &ws, &custom_object_input("Vendor", "Vendors"), admin).unwrap();
    let contract = custom_object_service::create(&conn, &ws, &custom_object_input("Supply Contract", "Supply Contracts"), admin).unwrap();

    relationship_service::create(&conn, &ws, &relationship_input(&contract.key, &vendor.key, false), admin).unwrap();

    let contract_detail = ontology_service::get_object_type_detail(&conn, &ws, &contract.key).unwrap();
    assert_eq!(contract_detail.link_types.len(), 1);
    let from_contract = &contract_detail.link_types[0];
    assert_eq!(from_contract.name, "Owning Vendor", "viewed from the source side, name is the forward label");
    assert_eq!(from_contract.inverse_name, "Managed Contracts");
    assert_eq!(from_contract.to_object_type.as_deref(), Some(vendor.key.as_str()));

    let vendor_detail = ontology_service::get_object_type_detail(&conn, &ws, &vendor.key).unwrap();
    assert_eq!(vendor_detail.link_types.len(), 1);
    let from_vendor = &vendor_detail.link_types[0];
    assert_eq!(from_vendor.name, "Managed Contracts", "viewed from the target side, name is the reverse label");
    assert_eq!(from_vendor.inverse_name, "Owning Vendor");
    assert_eq!(from_vendor.to_object_type.as_deref(), Some(contract.key.as_str()));

    // A polymorphic-target relationship has no single "to" type to name,
    // viewed from its own source side.
    relationship_service::create(&conn, &ws, &relationship_input(&contract.key, &vendor.key, true), admin).unwrap();
    let contract_detail = ontology_service::get_object_type_detail(&conn, &ws, &contract.key).unwrap();
    let polymorphic_link = contract_detail.link_types.iter().find(|l| l.to_object_type.is_none());
    assert!(polymorphic_link.is_some(), "the polymorphic relationship's source side must list with no fixed target type");
}

#[test]
fn get_object_type_detail_rejects_an_unknown_object_type() {
    let (conn, ws, _admin) = setup_workspace("Ontology Co");
    let result = ontology_service::get_object_type_detail(&conn, &ws, "not_a_real_object_type");
    assert!(result.is_err());
}
