//! Voice-First Mode PR 3 (GitHub issue #161): industry voice vocabulary
//! packs. Installs the real Property Management reference package and
//! confirms a spoken command resolves against *its own* status vocabulary
//! (Unit's "Occupancy Status" select field: Vacant/Reserved/Occupied/
//! Maintenance/Inactive) rather than only the generic Active/Inactive/
//! Archived column every Custom Object shares - and that the write lands
//! on that specific custom field (via `custom_field_service::set_entity_values`,
//! the same seam an existing record's own field edit already uses), not
//! the generic column, with real Undo back to the pre-command value.

use lanesra_core::db::open_in_memory_db;
use lanesra_core::models::access_role::{AccessRoleGrantInput, AccessRoleInput};
use lanesra_core::models::custom_record::CustomRecordInput;
use lanesra_core::models::industry_package::ImportPackageInput;
use lanesra_core::models::user::NewUser;
use lanesra_core::models::voice::{ConfirmVoicePlanInput, SetVoicePinInput, VoicePolicyBindingInput};
use lanesra_core::models::workspace::WorkspaceSetup;
use lanesra_core::repositories::custom_field_repo;
use lanesra_core::services::reference_packages::{lanesra_industry_foundation_manifest_json, property_management_manifest_json};
use lanesra_core::models::app_definition::AppPermissionInput;
use lanesra_core::services::{
    access_role_service, app_service, custom_field_service, custom_record_service, industry_package_service, user_service, voice_execution_service,
    voice_policy_service, voice_session_service, workspace_service,
};

fn master_key() -> [u8; 32] {
    [31u8; 32]
}

fn setup_workspace() -> (rusqlite::Connection, String, String) {
    let conn = open_in_memory_db().unwrap();
    let setup = WorkspaceSetup {
        business_name: "Acme Property Group".into(),
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

/// Installs the real Property Management reference package (and its real,
/// enforced Foundation dependency) - the same install path a real
/// workspace admin would use from Admin -> Industry Apps, not a
/// voice-specific fixture. Also grants the "Sales" legacy role Editor
/// access on every App Builder app the install creates (installing a
/// package publishes its own App claiming its objects - see
/// `app_service::require_object_write_access`'s own doc comment) - a real
/// non-Administrator voice user needs this same grant an admin would set
/// up from Admin -> App Builder before they could write these records at
/// all, voice or not.
fn install_property_management(conn: &rusqlite::Connection, ws: &str, admin: &str) {
    let foundation = ImportPackageInput { manifest_json: lanesra_industry_foundation_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &foundation, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap();
    let input = ImportPackageInput { manifest_json: property_management_manifest_json() };
    let package = industry_package_service::import_package(conn, ws, &input, Some(admin)).unwrap();
    industry_package_service::install(conn, ws, &package.id, Some(admin)).unwrap();
    for app in app_service::list(conn, ws).unwrap() {
        app_service::grant_permission(conn, &app.id, &AppPermissionInput { principal_type: "role".into(), principal_id: "Sales".into(), level: "editor".into() }, Some(admin)).unwrap();
    }
}

fn full_voice_access() -> VoicePolicyBindingInput {
    VoicePolicyBindingInput {
        access_role_id: None,
        can_use_voice: true,
        can_search: true,
        can_create: true,
        can_update: true,
        can_act: true,
        can_bulk_act: true,
        can_external_act: true,
        can_use_agents: true,
        max_action_level: "act_with_confirmation".into(),
        processing_boundary: "cloud".into(),
        max_unlock_minutes: 30,
    }
}

fn unlocked_session(conn: &rusqlite::Connection, ws: &str, admin: &str, username: &str) -> (String, String) {
    let user = user_service::create(
        conn,
        ws,
        &NewUser { username: username.into(), display_name: username.into(), password: "anothersecretpw".into(), roles: vec!["Sales".to_string()] },
        Some(admin),
    )
    .unwrap();
    let role = access_role_service::create(conn, ws, &AccessRoleInput { name: format!("{username}-role"), description: "".into() }, Some(admin)).unwrap();
    access_role_service::upsert_grant(
        conn,
        &role.id,
        &AccessRoleGrantInput { object_key: "*".into(), can_create: true, can_read: true, can_update: true, can_delete: true, can_assign: true, record_scope: "ORGANIZATION".into() },
        Some(admin),
    )
    .unwrap();
    access_role_service::assign_to_user(conn, &user.id, &role.id, Some(admin)).unwrap();
    voice_policy_service::upsert_policy_binding(conn, ws, Some(admin), &VoicePolicyBindingInput { access_role_id: Some(role.id), ..full_voice_access() }).unwrap();
    voice_session_service::set_pin(conn, &user.id, &SetVoicePinInput { pin: "1234".into() }).unwrap();
    let session = voice_session_service::unlock(conn, &user.id, "1234").unwrap();
    (user.id, session.id)
}

#[tokio::test]
async fn spoken_status_resolves_against_an_installed_industry_apps_own_vocabulary() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);
    let unit = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: "unit".into(), primary_name: "Riverside 5A".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    // Seeds Unit's own other required field (Unit Number, no default) the
    // same way the real record form would have at creation - otherwise
    // `set_entity_values`'s required-field validation (which always checks
    // every field on the object, not just the one being changed) would
    // reject this test's voice-driven Occupancy Status update for an
    // unrelated reason. Deliberately a different string from the primary
    // name above - both being searchable, an identical value would surface
    // the same record twice (once per matching field), an unrelated
    // resolver-level dedup gap this test isn't about.
    custom_field_service::set_entity_values(&conn, "unit", &unit.id, &[("unit_number".to_string(), "UN-1205".to_string())].into_iter().collect(), Some(&admin)).unwrap();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "leasing-agent");

    // "Occupied" isn't one of the generic Active/Inactive/Archived values
    // every Custom Object shares - it only exists as one of Property
    // Management's own Unit "Occupancy Status" options, so this only
    // resolves at all if the planner reads that installed package's real
    // field metadata, not just the fixed CUSTOM_RECORD_STATUSES list.
    let outcome = voice_execution_service::submit_command(&conn, &session_id, &user, "mark unit Riverside 5A as Occupied", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("a real installed Industry App's own status vocabulary must be voice-addressable");
    assert_eq!(plan.status, "awaiting_confirmation", "update_status is Medium risk and this policy always confirms above Low");

    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");

    let values = custom_field_repo::get_values(&conn, &unit.id).unwrap();
    assert_eq!(values.get("unit_stage").map(|s| s.as_str()), Some("Occupied"), "must write the specific Occupancy Status field, not the generic status column");
    let record = custom_record_service::get(&conn, &unit.id).unwrap();
    assert_eq!(record.status, "Active", "the generic Active/Inactive/Archived column must be left untouched - the value only ever belonged to the industry field");

    // Real Undo: reverts unit_stage back to whatever it was before this
    // command ("Vacant" - its own configured default_value, already
    // applied when Unit Number was seeded above), never just leaving the
    // new "Occupied" value in place.
    let execution_id = result.executions[0].id.clone();
    voice_execution_service::undo(&conn, &execution_id, &admin).unwrap();
    let values_after_undo = custom_field_repo::get_values(&conn, &unit.id).unwrap();
    assert_eq!(values_after_undo.get("unit_stage").map(|s| s.as_str()), Some("Vacant"), "undo must restore the pre-command value, not just leave the new one in place");
}

#[tokio::test]
async fn a_value_shared_with_the_generic_column_still_uses_the_generic_column() {
    let (conn, ws, admin) = setup_workspace();
    install_property_management(&conn, &ws, &admin);
    let unit = custom_record_service::create(
        &conn,
        &ws,
        &CustomRecordInput { object_key: "unit".into(), primary_name: "Maple 9C".into(), status: "Active".into(), owner_user_id: None, notes: None },
        Some(&admin),
    )
    .unwrap();
    custom_field_service::set_entity_values(&conn, "unit", &unit.id, &[("unit_number".to_string(), "UN-2077".to_string())].into_iter().collect(), Some(&admin)).unwrap();
    let (user, session_id) = unlocked_session(&conn, &ws, &admin, "leasing-agent-2");

    // "Inactive" is a real option on both the generic status column and
    // Unit's own Occupancy Status field - the generic column must still
    // win (checked first), exactly like every other Custom Object without
    // any installed industry vocabulary at all.
    let outcome = voice_execution_service::submit_command(&conn, &session_id, &user, "mark unit Maple 9C as Inactive", "en-US", None, &master_key()).await.unwrap();
    let plan = outcome.plan.expect("Inactive is still a valid generic status value");
    let result = voice_execution_service::confirm_plan(&conn, &session_id, &user, &ConfirmVoicePlanInput { plan_id: plan.id, method: "tap".into(), edited_plan: None }, &master_key()).await.unwrap();
    assert_eq!(result.status, "succeeded");

    let record = custom_record_service::get(&conn, &unit.id).unwrap();
    let values = custom_field_repo::get_values(&conn, &unit.id).unwrap();
    assert_eq!(record.status, "Inactive");
    assert_eq!(values.get("unit_stage").map(|s| s.as_str()), Some("Vacant"), "the industry field must be untouched (still its own configured default) when the generic column already satisfied the spoken value");
}
