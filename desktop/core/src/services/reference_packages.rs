//! Reference industry packages - real manifests written against
//! `models::industry_package`/`industry_package_service`, kept in Rust
//! (not shipped as loose files) so they're compiled, testable, and
//! versioned with the engine they target. The dev spec ("Top 10 Industry
//! Data Models & Packaged Business Apps") sequences Field Service and
//! Property Management first; this module ships Field Service as the
//! first one, proving the foundation against real content rather than
//! only the synthetic manifests `industry_data_model.rs`'s tests use.
//!
//! Where the spec calls for something the current engine genuinely can't
//! express, that item is left out rather than faked - each gap is called
//! out below, at the exact point it's skipped, so it's easy to revisit
//! once the underlying engine gains the capability:
//! - Cross-record validation (a business rule reading a *related*
//!   record's own field, e.g. "the selected Asset must belong to the
//!   selected Site") - conditions only ever see the triggering record's
//!   own field values.
//! - `date_reached`/`due_overdue` triggers on a custom object - a
//!   workflow trigger's watchable date fields are one specific, hardcoded
//!   set of core-entity fields (`models::workflow::date_fields_for`),
//!   empty for every custom object.
//! - A per-object custom status/stage vocabulary using the built-in
//!   `status_changed` trigger or `status`/`stage` action targets - every
//!   custom object's built-in status is the fixed Active/Inactive/
//!   Archived set (`CUSTOM_RECORD_STATUSES`). This package works around
//!   it the intended way: an ordinary custom select field (`stage` on
//!   Work Order, `appt_stage` on Service Appointment) carries the real
//!   domain vocabulary, driving `field_changed`-triggered workflows and
//!   `field_source: "custom"` rule conditions exactly like any other
//!   custom field would.
//! - A workflow triggered by two records getting *linked* via
//!   `relationship_service::link` - there's no such trigger type
//!   (`record_created`/`record_updated`/`field_changed`/... all watch one
//!   record's own save, not a separate link action against it); see
//!   `field_service_workflows`'s own doc comment for where this ruled out
//!   an otherwise-natural automation.

use serde_json::json;

/// `lanesra.field_service` v1.0.0 - see this module's own doc comment for
/// what's included and what's deliberately left out. Returns the raw
/// manifest JSON text, the same shape `ImportPackageInput::manifest_json`
/// expects, so a caller can either import it as-is or show it to an
/// admin for review first (the Admin -> App Catalog screen's "Load
/// starter package" button does the latter, matching the spec's own
/// Review -> Validate -> Install flow instead of a silent one-click
/// install).
pub fn field_service_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.field_service",
        "name": "Field Service",
        "industry": "Field Service",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "service_site", "singular_label": "Service Site", "plural_label": "Service Sites", "icon": "📍", "prefix": "SITE", "digits": 4 },
            { "key": "asset", "singular_label": "Asset", "plural_label": "Assets", "icon": "🔧", "prefix": "AST", "digits": 5 },
            { "key": "work_type", "singular_label": "Work Type", "plural_label": "Work Types", "icon": "🗂", "prefix": "WT", "digits": 3 },
            { "key": "work_order", "singular_label": "Work Order", "plural_label": "Work Orders", "icon": "🧰", "prefix": "WO", "digits": 5 },
            { "key": "work_order_line", "singular_label": "Work Order Line", "plural_label": "Work Order Lines", "icon": "📄", "prefix": "WOL", "digits": 5 },
            { "key": "service_appointment", "singular_label": "Service Appointment", "plural_label": "Service Appointments", "icon": "📅", "prefix": "APT", "digits": 5 },
            { "key": "resource_profile", "singular_label": "Resource Profile", "plural_label": "Resource Profiles", "icon": "🧑‍🔧", "prefix": "RES", "digits": 4 },
            { "key": "skill", "singular_label": "Skill", "plural_label": "Skills", "icon": "🎓", "prefix": "SKL", "digits": 3 },
            { "key": "service_territory", "singular_label": "Service Territory", "plural_label": "Service Territories", "icon": "🗺", "prefix": "TER", "digits": 3 }
        ],
        "fields": field_service_fields(),
        "relationships": field_service_relationships(),
        "business_rules": field_service_business_rules(),
        "workflows": field_service_workflows(),
        "screen_layouts": [
            {
                "entity_type": "work_order",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                {
                                    "id": "overview",
                                    "title": "Overview",
                                    "columns": 2,
                                    "fields": ["description", "stage", "priority", "requested_date", "resolution", "completion_date"]
                                }
                            ],
                            // Indices into `relationships` below: Asset (4), Appointments (6), Lines (5) -
                            // resolved to real relationship keys by the installer.
                            "related": ["4", "6", "5"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Work Orders by Stage", "entity_type": "work_order", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Field Service Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Field Service",
            "icon": "🔧",
            "description": "Dispatch, work orders, assets and service appointments for on-site service businesses.",
            "object_keys": [
                "work_order", "service_appointment", "asset", "service_site", "work_order_line",
                "work_type", "resource_profile", "skill", "service_territory", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // Roles are recommendations only (spec: "always reviewed by
            // administrator before activation") - mapped onto this
            // build's actual role set (Administrator/Manager/Sales/
            // Finance/ReadOnly; see `user_repo::ROLES`), not the spec's
            // own Dispatcher/Technician/Service Manager labels, which
            // don't exist as roles in this platform.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        "seed_data": [
            {
                "object_key": "work_type",
                "record": { "object_key": "work_type", "primary_name": "Standard HVAC Service Call", "status": "Active", "owner_user_id": null, "notes": null },
                "field_values": { "estimated_duration_minutes": "90" }
            },
            {
                "object_key": "skill",
                "record": { "object_key": "skill", "primary_name": "HVAC Certified", "status": "Active", "owner_user_id": null, "notes": null },
                "field_values": { "category": "Technical" }
            },
            {
                "object_key": "service_territory",
                "record": { "object_key": "service_territory", "primary_name": "Downtown", "status": "Active", "owner_user_id": null, "notes": null },
                "field_values": { "geography": "City Center" }
            }
        ]
    })
    .to_string()
}

/// Split into one `json!` array per object (rather than one array across
/// all ~30 field definitions) purely to stay under the `json!` macro's
/// recursion limit - no semantic significance to the grouping beyond
/// that.
fn field_service_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        service_site_fields(),
        asset_fields(),
        work_type_fields(),
        work_order_fields(),
        work_order_line_fields(),
        service_appointment_fields(),
        resource_profile_fields(),
        skill_fields(),
        service_territory_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn service_site_fields() -> serde_json::Value {
    json!([
        { "key": "address", "entity_type": "service_site", "label": "Address", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "access_notes", "entity_type": "service_site", "label": "Access Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Gate codes, parking, pets, etc.", "placeholder": null },
        { "key": "timezone", "entity_type": "service_site", "label": "Timezone", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "service_window", "entity_type": "service_site", "label": "Service Window", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "e.g. Mon-Fri 9am-5pm", "placeholder": null }
    ])
}

fn asset_fields() -> serde_json::Value {
    json!([
        { "key": "serial_number", "entity_type": "asset", "label": "Serial Number", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": true, "help_text": null, "placeholder": null },
        { "key": "model", "entity_type": "asset", "label": "Model", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "manufacturer", "entity_type": "asset", "label": "Manufacturer", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "install_date", "entity_type": "asset", "label": "Install Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "warranty_expiry", "entity_type": "asset", "label": "Warranty Expiry", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Written by the "Work completed" workflow (see field_service_workflows) - copied from
        // the completing Work Order's own completion_date, rather than a literal "today" the
        // workflow engine has no way to express (see this module's own doc comment).
        { "key": "last_service_date", "entity_type": "asset", "label": "Last Service Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "next_service_date", "entity_type": "asset", "label": "Next Service Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Preventive-maintenance scheduling is manual for now - see this module's own doc comment on date_reached triggers.", "placeholder": null },
        { "key": "asset_stage", "entity_type": "asset", "label": "Asset Status", "field_type": "select", "options": ["Active", "Out of Service", "Retired"], "required": true, "show_in_list": true, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn work_type_fields() -> serde_json::Value {
    json!([
        { "key": "estimated_duration_minutes", "entity_type": "work_type", "label": "Estimated Duration (minutes)", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "required_skills", "entity_type": "work_type", "label": "Required Skills", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Comma-separated skill names", "placeholder": null }
    ])
}

fn work_order_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "work_order", "label": "Stage", "field_type": "select", "options": ["New", "Scheduled", "In Progress", "On Hold", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "New", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "priority", "entity_type": "work_order", "label": "Priority", "field_type": "select", "options": ["Low", "Medium", "High", "Urgent"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Medium", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "requested_date", "entity_type": "work_order", "label": "Requested Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description", "entity_type": "work_order", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed by the "Completion validation" business rule below.
        { "key": "resolution", "entity_type": "work_order", "label": "Resolution", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "completion_date", "entity_type": "work_order", "label": "Completion Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn work_order_line_fields() -> serde_json::Value {
    json!([
        { "key": "quantity", "entity_type": "work_order_line", "label": "Quantity", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "1", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "unit_price", "entity_type": "work_order_line", "label": "Unit Price", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "work_order_line", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "actual_quantity", "entity_type": "work_order_line", "label": "Actual Quantity", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_appointment_fields() -> serde_json::Value {
    json!([
        { "key": "appt_stage", "entity_type": "service_appointment", "label": "Stage", "field_type": "select", "options": ["Unscheduled", "Scheduled", "En Route", "On Site", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Unscheduled", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "scheduled_start", "entity_type": "service_appointment", "label": "Scheduled Start", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "scheduled_end", "entity_type": "service_appointment", "label": "Scheduled End", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "actual_start", "entity_type": "service_appointment", "label": "Actual Start", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "actual_end", "entity_type": "service_appointment", "label": "Actual End", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "outcome", "entity_type": "service_appointment", "label": "Outcome", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "travel_notes", "entity_type": "service_appointment", "label": "Travel Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn resource_profile_fields() -> serde_json::Value {
    json!([
        { "key": "capacity_hours_per_week", "entity_type": "resource_profile", "label": "Capacity (hours/week)", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": "168", "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "resource_active", "entity_type": "resource_profile", "label": "Active", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn skill_fields() -> serde_json::Value {
    json!([
        { "key": "category", "entity_type": "skill", "label": "Category", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_territory_fields() -> serde_json::Value {
    json!([
        { "key": "geography", "entity_type": "service_territory", "label": "Geography", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: `screen_layouts[0].draft`'s `related`
/// and both `update_related_record` workflow actions reference these
/// relationships by their position in this array (see this module's own
/// doc comment / `IndustryPackageManifest::workflows`' doc comment).
fn field_service_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "service_site", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Company", "reverse_label": "Service Sites", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "asset", "target_entity_type": "service_site", "relationship_type": "many_to_one", "forward_label": "Service Site", "reverse_label": "Assets", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "work_order", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Work Orders", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 3 */ { "source_entity_type": "work_order", "target_entity_type": "service_site", "relationship_type": "many_to_one", "forward_label": "Service Site", "reverse_label": "Work Orders", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 4 */ { "source_entity_type": "work_order", "target_entity_type": "asset", "relationship_type": "many_to_one", "forward_label": "Asset", "reverse_label": "Work Orders", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 5 */ { "source_entity_type": "work_order_line", "target_entity_type": "work_order", "relationship_type": "many_to_one", "forward_label": "Work Order", "reverse_label": "Lines", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 6 */ { "source_entity_type": "service_appointment", "target_entity_type": "work_order", "relationship_type": "many_to_one", "forward_label": "Work Order", "reverse_label": "Appointments", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 7 */ { "source_entity_type": "resource_profile", "target_entity_type": "skill", "relationship_type": "many_to_many", "forward_label": "Skills", "reverse_label": "Resource Profiles", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 8 */ { "source_entity_type": "resource_profile", "target_entity_type": "service_territory", "relationship_type": "many_to_many", "forward_label": "Territories", "reverse_label": "Resource Profiles", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 9 */ { "source_entity_type": "work_order", "target_entity_type": "Contract", "relationship_type": "many_to_one", "forward_label": "Contract", "reverse_label": "Work Orders", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 10 */ { "source_entity_type": "work_order_line", "target_entity_type": "Product", "relationship_type": "many_to_one", "forward_label": "Product / Service", "reverse_label": "Work Order Lines", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 }
    ])
}

/// Only the two spec rules a same-record condition/action engine can
/// actually express - "Asset/site integrity" and "Warranty warning" both
/// need to read a *related* record's own fields, which conditions can't
/// do; see this module's own doc comment.
fn field_service_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "work_order",
            "name": "Completion validation",
            "description": "A completed work order must record what was resolved and when.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completion_date", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "resolution", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "service_appointment",
            "name": "Appointment completion",
            "description": "A completed appointment must record its actual times and outcome.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "appt_stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "actual_start", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "actual_end", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "outcome", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// Two of the spec's five workflows. Left out, beyond the two already
/// noted in this module's own doc comment:
/// - "Technician departure" - no activity-log action exists to write to.
/// - "Preventive maintenance" - needs a `date_reached` trigger on a
///   custom object's own date field, which the engine doesn't support.
/// - The spec's "Appointment created -> update its Work Order to
///   Scheduled" - a workflow can only trigger on `record_created` (the
///   instant a record is saved) or on a field changing, and linking a
///   record to another via `relationship_service::link` is a *separate*,
///   later action (the related-list "Link" flow) with no workflow
///   trigger of its own. A `record_created` workflow on Service
///   Appointment always fires before any such link exists, so
///   `update_related_record` would find nothing to update, every time -
///   not a useful automation. "Work completed updates asset" below
///   avoids this because linking an Asset to its Work Order naturally
///   happens well before that Work Order is later marked Completed.
fn field_service_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "work_order",
            "name": "New work order created",
            "description": "Give the dispatcher a task to schedule it and let the service manager know.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Schedule this work order\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" },
                { "action_type": "add_notification", "params_json": "{\"message\":\"New work order created\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "work_order",
            "name": "Work completed updates asset",
            "description": "Completing a work order records its completion date onto the linked asset's service history.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":4,\"target_field_key\":\"last_service_date\",\"target_field_source\":\"custom\",\"value\":null,\"copy_from_field_key\":\"completion_date\"}" }
            ]
        }
    ])
}
