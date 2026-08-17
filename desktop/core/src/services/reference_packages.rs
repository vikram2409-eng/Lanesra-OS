//! Reference industry packages - real manifests written against
//! `models::industry_package`/`industry_package_service`, kept in Rust
//! (not shipped as loose files) so they're compiled, testable, and
//! versioned with the engine they target. The dev spec ("Top 10 Industry
//! Data Models & Packaged Business Apps") sequences Field Service,
//! Property Management and Construction & Contractors first; this module
//! ships all three, proving the foundation against real content rather
//! than only the synthetic manifests `industry_data_model.rs`'s tests use.
//!
//! Where the spec calls for something the current engine genuinely can't
//! express, that item is left out rather than faked - each gap is called
//! out below, at the exact point it's skipped, so it's easy to revisit
//! once the underlying engine gains the capability:
//! - Cross-record validation (a business rule reading a *related*
//!   record's own field, e.g. "the selected Asset must belong to the
//!   selected Site", or Property Management's "the Unit must not already
//!   have an overlapping active Lease") - conditions only ever see the
//!   triggering record's own field values, never a related record's or an
//!   aggregate across several.
//! - `date_reached`/`due_overdue` triggers on a custom object - a
//!   workflow trigger's watchable date fields are one specific, hardcoded
//!   set of core-entity fields (`models::workflow::date_fields_for`),
//!   empty for every custom object. Rules out Field Service's preventive-
//!   maintenance workflow and Property Management's lease-renewal/
//!   document-expiry ones.
//! - A per-object custom status/stage vocabulary using the built-in
//!   `status_changed` trigger or `status`/`stage` action targets - every
//!   custom object's built-in status is the fixed Active/Inactive/
//!   Archived set (`CUSTOM_RECORD_STATUSES`). Both packages work around
//!   it the intended way: an ordinary custom select field (`stage` on
//!   Work Order, `appt_stage` on Service Appointment, `stage` on Lease,
//!   `unit_stage` on Unit, ...) carries the real domain vocabulary,
//!   driving `field_changed`-triggered workflows and
//!   `field_source: "custom"` rule conditions exactly like any other
//!   custom field would.
//! - A workflow triggered by two records getting *linked* via
//!   `relationship_service::link` - there's no such trigger type
//!   (`record_created`/`record_updated`/`field_changed`/... all watch one
//!   record's own save, not a separate link action against it); see
//!   `field_service_workflows`'s own doc comment for where this ruled out
//!   an otherwise-natural automation.
//! - Conditional, install-time-optional content ("only create this
//!   relationship/screen if another package is installed") - the
//!   manifest format has no such conditional; Property Management's own
//!   optional Field Service integration (spec: "Maintenance Request 1:1
//!   optional Field Service Work Order") is left out entirely rather than
//!   shipped as a relationship to an object that may not exist, matching
//!   the spec's own "must work without it" requirement the honest way -
//!   by not depending on it at all yet.
//! - An accumulating/incrementing workflow action - `update_related_record`
//!   only ever *sets* a related record's field to a literal or a value
//!   copied from the triggering record, it never reads the target's
//!   current value first. Construction & Contractors' "Change approved
//!   increments Project's approved change value" is left unautomated for
//!   this reason (see `construction_workflows`'s own doc comment) rather
//!   than shipped as an overwrite that would silently erase every earlier
//!   approval's contribution instead of accumulating them.
//! - A conditional bulk update through a relationship - `update_related_record`
//!   writes to *every* record currently linked through the named
//!   relationship, with no way to filter which ones by another field on
//!   those records. Construction & Contractors' "Project close closes
//!   outstanding tasks that are configured auto-close" needs exactly that
//!   filter (only the flagged tasks, not every task on the project) and
//!   is left out for the same reason.

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

// --- lanesra.property_management -------------------------------------

/// `lanesra.property_management` v1.0.0 - see this module's own doc
/// comment for what's included and deliberately left out (notably: no
/// Field Service integration, no lease-overlap/occupancy cross-record
/// checks, no lease-renewal or document-expiry date-based workflows).
pub fn property_management_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.property_management",
        "name": "Property Management",
        "industry": "Property Management",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "property", "singular_label": "Property", "plural_label": "Properties", "icon": "🏢", "prefix": "PROP", "digits": 4 },
            { "key": "unit", "singular_label": "Unit", "plural_label": "Units", "icon": "🚪", "prefix": "UNIT", "digits": 5 },
            { "key": "lease", "singular_label": "Lease", "plural_label": "Leases", "icon": "📃", "prefix": "LSE", "digits": 5 },
            // Junction object rather than a plain many_to_many relationship, since a party's
            // "role" (Tenant/Guarantor/Occupant) has nowhere to live on a bare RelationshipInstance.
            { "key": "lease_party", "singular_label": "Lease Party", "plural_label": "Lease Parties", "icon": "🧑", "prefix": "LP", "digits": 5 },
            { "key": "rent_schedule", "singular_label": "Rent Schedule Entry", "plural_label": "Rent Schedule", "icon": "💰", "prefix": "RENT", "digits": 6 },
            { "key": "maintenance_request", "singular_label": "Maintenance Request", "plural_label": "Maintenance Requests", "icon": "🛠", "prefix": "MR", "digits": 5 },
            { "key": "vendor_assignment", "singular_label": "Vendor Assignment", "plural_label": "Vendor Assignments", "icon": "🧰", "prefix": "VA", "digits": 5 },
            { "key": "property_document", "singular_label": "Property Document", "plural_label": "Property Documents", "icon": "📄", "prefix": "DOC", "digits": 5 }
        ],
        "fields": property_management_fields(),
        "relationships": property_management_relationships(),
        "business_rules": property_management_business_rules(),
        "workflows": property_management_workflows(),
        "screen_layouts": [
            {
                "entity_type": "lease",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "terms",
                            "title": "Terms",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["stage", "start_date", "end_date", "rent_amount", "deposit_amount"] }
                            ],
                            // Indices into `relationships` below: Lease Parties (3), Rent Schedule (5).
                            "related": ["3", "5"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Leases by Stage", "entity_type": "lease", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Property Management Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Property Management",
            "icon": "🏘",
            "description": "Properties, units, leases, rent schedules and maintenance for residential/commercial property managers.",
            "object_keys": [
                "property", "unit", "lease", "lease_party", "rent_schedule",
                "maintenance_request", "vendor_assignment", "property_document", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names (Property Manager, Leasing, Maintenance Coordinator, ...) onto
            // this build's actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "editor" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object exists in this data model the way Field
        // Service's Skill/Territory did (every object here is a live business
        // record) - nothing to seed under spec 1.1's "reference data only".
        "seed_data": []
    })
    .to_string()
}

fn property_management_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        property_fields(),
        unit_fields(),
        lease_fields(),
        lease_party_fields(),
        rent_schedule_fields(),
        maintenance_request_fields(),
        vendor_assignment_fields(),
        property_document_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn property_fields() -> serde_json::Value {
    json!([
        { "key": "property_type", "entity_type": "property", "label": "Property Type", "field_type": "select", "options": ["Residential", "Commercial", "Mixed Use"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "address", "entity_type": "property", "label": "Address", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "property", "label": "Status", "field_type": "select", "options": ["Active", "Inactive", "Sold / No Longer Managed"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn unit_fields() -> serde_json::Value {
    json!([
        { "key": "unit_number", "entity_type": "unit", "label": "Unit Number", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "unit_type", "entity_type": "unit", "label": "Unit Type", "field_type": "select", "options": ["Studio", "1BR", "2BR", "3BR+", "Commercial"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "size_sqft", "entity_type": "unit", "label": "Size (sqft)", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "bedrooms", "entity_type": "unit", "label": "Bedrooms", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "rent_target", "entity_type": "unit", "label": "Rent Target", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Written by the Lease activation / termination workflows below.
        { "key": "unit_stage", "entity_type": "unit", "label": "Occupancy Status", "field_type": "select", "options": ["Vacant", "Reserved", "Occupied", "Maintenance", "Inactive"], "required": true, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Vacant", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn lease_fields() -> serde_json::Value {
    json!([
        { "key": "start_date", "entity_type": "lease", "label": "Start Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "lease", "label": "End Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "rent_amount", "entity_type": "lease", "label": "Rent Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "deposit_amount", "entity_type": "lease", "label": "Deposit Amount", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "lease", "label": "Stage", "field_type": "select", "options": ["Draft", "Pending Signature", "Active", "Expiring", "Renewed", "Expired", "Terminated"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn lease_party_fields() -> serde_json::Value {
    json!([
        { "key": "role", "entity_type": "lease_party", "label": "Role", "field_type": "select", "options": ["Tenant", "Guarantor", "Occupant"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Tenant", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn rent_schedule_fields() -> serde_json::Value {
    json!([
        { "key": "due_date", "entity_type": "rent_schedule", "label": "Due Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "frequency", "entity_type": "rent_schedule", "label": "Frequency", "field_type": "select", "options": ["Monthly", "Quarterly", "Annual"], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "Monthly", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expected_amount", "entity_type": "rent_schedule", "label": "Expected Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "schedule_stage", "entity_type": "rent_schedule", "label": "Status", "field_type": "select", "options": ["Pending", "Paid", "Overdue", "Waived"], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn maintenance_request_fields() -> serde_json::Value {
    json!([
        { "key": "category", "entity_type": "maintenance_request", "label": "Category", "field_type": "select", "options": ["Plumbing", "Electrical", "HVAC", "Appliance", "General", "Other"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "priority", "entity_type": "maintenance_request", "label": "Priority", "field_type": "select", "options": ["Low", "Medium", "High", "Urgent"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Medium", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description", "entity_type": "maintenance_request", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "maintenance_request", "label": "Stage", "field_type": "select", "options": ["New", "Assigned", "In Progress", "Waiting", "Resolved", "Closed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "New", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Closed by the "Maintenance closure" business rule below.
        { "key": "resolution", "entity_type": "maintenance_request", "label": "Resolution", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "completed_date", "entity_type": "maintenance_request", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn vendor_assignment_fields() -> serde_json::Value {
    json!([
        { "key": "assigned_date", "entity_type": "vendor_assignment", "label": "Assigned Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "notes", "entity_type": "vendor_assignment", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn property_document_fields() -> serde_json::Value {
    json!([
        { "key": "category", "entity_type": "property_document", "label": "Category", "field_type": "select", "options": ["Lease Agreement", "Insurance", "Inspection", "Compliance", "Other"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "effective_date", "entity_type": "property_document", "label": "Effective Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expiry_date", "entity_type": "property_document", "label": "Expiry Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: `screen_layouts[0].draft`'s `related`
/// and both `update_related_record` workflow actions reference these
/// relationships by their position in this array.
fn property_management_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "property", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Owner", "reverse_label": "Properties", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "unit", "target_entity_type": "property", "relationship_type": "many_to_one", "forward_label": "Property", "reverse_label": "Units", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "lease", "target_entity_type": "unit", "relationship_type": "many_to_one", "forward_label": "Unit", "reverse_label": "Leases", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "lease_party", "target_entity_type": "lease", "relationship_type": "many_to_one", "forward_label": "Lease", "reverse_label": "Parties", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "lease_party", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Contact", "reverse_label": "Lease Roles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 5 */ { "source_entity_type": "rent_schedule", "target_entity_type": "lease", "relationship_type": "many_to_one", "forward_label": "Lease", "reverse_label": "Rent Schedule", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "maintenance_request", "target_entity_type": "property", "relationship_type": "many_to_one", "forward_label": "Property", "reverse_label": "Maintenance Requests", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 7 */ { "source_entity_type": "maintenance_request", "target_entity_type": "unit", "relationship_type": "many_to_one", "forward_label": "Unit", "reverse_label": "Maintenance Requests", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 8 */ { "source_entity_type": "vendor_assignment", "target_entity_type": "maintenance_request", "relationship_type": "many_to_one", "forward_label": "Maintenance Request", "reverse_label": "Vendor Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 9 */ { "source_entity_type": "vendor_assignment", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Vendor", "reverse_label": "Vendor Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 10 */ { "source_entity_type": "property_document", "target_entity_type": "property", "relationship_type": "many_to_one", "forward_label": "Property", "reverse_label": "Documents", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 3 }
    ])
}

/// Two of the spec's four rules - "Occupancy conflict" and "Lease
/// activation"'s own eligibility checks both need cross-record/aggregate
/// reads (another Lease on the same Unit; counting linked Lease Parties)
/// no condition can do; see this module's own doc comment.
fn property_management_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "lease",
            "name": "Lease date validation",
            "description": "A lease's end date must be after its start date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "end_date", "operator": "on_or_before", "value": "", "compare_field_source": "custom", "compare_field_key": "start_date" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Lease end date must be after the start date." }
            ]
        },
        {
            "entity_type": "maintenance_request",
            "name": "Maintenance closure",
            "description": "A closed maintenance request must record its resolution and completion date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "resolution", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "completed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// Three of the spec's five workflows - "Lease renewal" and "Document
/// expiry" both need a `date_reached` trigger on a custom object, which
/// the engine doesn't support (see this module's own doc comment).
/// "Maintenance intake"'s optional Work Order creation is left out for
/// the same reason the Field Service relationship itself is: no
/// conditional-on-another-package's-presence content in the manifest
/// format yet. "Lease termination/expiry" simplifies away the spec's
/// "unless another active lease exists" clause - checking that needs
/// reading every other Lease linked to the same Unit, a cross-record
/// aggregate no condition can express.
fn property_management_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "lease",
            "name": "Lease activation",
            "description": "Activating a lease marks its unit occupied.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Active" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":2,\"target_field_key\":\"unit_stage\",\"target_field_source\":\"custom\",\"value\":\"Occupied\",\"copy_from_field_key\":null}" }
            ]
        },
        {
            "entity_type": "lease",
            "name": "Lease termination or expiry",
            "description": "Ending a lease frees up its unit.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "in_list", "value": "Expired|Terminated" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":2,\"target_field_key\":\"unit_stage\",\"target_field_source\":\"custom\",\"value\":\"Vacant\",\"copy_from_field_key\":null}" }
            ]
        },
        {
            "entity_type": "maintenance_request",
            "name": "Maintenance intake",
            "description": "Give the maintenance coordinator a task as soon as a request comes in.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Triage this maintenance request\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.construction -----------------------------------------------

/// `lanesra.construction` v1.0.0 - the third package, sequenced right
/// after Property Management per the dev spec. See this module's own doc
/// comment for what's included and deliberately left out.
///
/// A few simplifications relative to the spec's own object model, beyond
/// the standard omissions this module's doc comment already covers:
/// - No separate Project Site object - the spec itself lists it as
///   "Custom or shared Site"; its two fields (address, access notes) are
///   folded straight onto Project instead of a second object.
/// - No separate Estimate object - the spec's own technical note says
///   "Estimate screen can extend Quote rather than create a parallel
///   quote engine", so Project relates to the core Quote object directly.
/// - "Project Manager" is the custom object's own built-in owner field,
///   not a duplicate custom field.
pub fn construction_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.construction",
        "name": "Construction & Contractors",
        "industry": "Construction",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "project", "singular_label": "Project", "plural_label": "Projects", "icon": "🏗", "prefix": "PROJ", "digits": 4 },
            { "key": "work_package", "singular_label": "Work Package", "plural_label": "Work Packages", "icon": "📦", "prefix": "WP", "digits": 4 },
            { "key": "change_order", "singular_label": "Change Order", "plural_label": "Change Orders", "icon": "📝", "prefix": "CO", "digits": 4 },
            { "key": "subcontract_assignment", "singular_label": "Subcontract Assignment", "plural_label": "Subcontract Assignments", "icon": "🤝", "prefix": "SUB", "digits": 4 },
            { "key": "inspection", "singular_label": "Inspection", "plural_label": "Inspections", "icon": "🔎", "prefix": "INSP", "digits": 4 }
        ],
        "fields": construction_fields(),
        "relationships": construction_relationships(),
        "business_rules": construction_business_rules(),
        "workflows": construction_workflows(),
        "screen_layouts": [
            {
                "entity_type": "project",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["stage", "start_date", "end_date", "actual_end_date", "contract_value", "approved_change_value"] },
                                { "id": "site", "title": "Site", "columns": 1, "fields": ["site_address", "site_access_notes"] }
                            ],
                            // Indices into `relationships` below: Work Packages (1), Change Orders (2), Inspections (5), Invoices (8).
                            "related": ["1", "2", "5", "8"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Projects by Stage", "entity_type": "project", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Construction Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Construction & Contractors",
            "icon": "🏗️",
            "description": "Projects, work packages, change orders, subcontractor assignments and inspections for general contractors and specialty trades.",
            "object_keys": [
                "project", "work_package", "change_order", "subcontract_assignment", "inspection", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names (Construction Admin, Project Manager, Estimator, Site Supervisor)
            // onto this build's actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object exists here either (same as Property
        // Management) - every object is a live project-operations record.
        "seed_data": []
    })
    .to_string()
}

fn construction_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        project_fields(),
        construction_opportunity_fields(),
        work_package_fields(),
        change_order_fields(),
        subcontract_assignment_fields(),
        inspection_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn project_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "project", "label": "Stage", "field_type": "select", "options": ["Lead/Estimating", "Awarded", "Planning", "Active", "On Hold", "Substantially Complete", "Closed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Lead/Estimating", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "site_address", "entity_type": "project", "label": "Site Address", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "site_access_notes", "entity_type": "project", "label": "Site Access Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Gate codes, parking, hazards, etc.", "placeholder": null },
        { "key": "start_date", "entity_type": "project", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "project", "label": "Planned End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Closed by the "Close project" business rule below.
        { "key": "actual_end_date", "entity_type": "project", "label": "Actual End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "contract_value", "entity_type": "project", "label": "Contract Value", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Manually maintained, not auto-incremented by the Change approval workflow -
        // see this module's own doc comment on the missing accumulate/increment action.
        { "key": "approved_change_value", "entity_type": "project", "label": "Approved Change Value", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 7, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": "0", "is_unique": false, "help_text": "Update this manually when a change order is approved - see the 'Change order approved' workflow's notification.", "placeholder": null }
    ])
}

/// A custom field on the *built-in* Opportunity entity, not on any custom
/// object here - custom fields work on any of the nine built-in entity
/// types as well as custom objects (see `custom_field_service`), which is
/// what makes the spec's "Opportunity Won AND 'Create Project' enabled"
/// condition on the Opportunity Won workflow expressible at all.
fn construction_opportunity_fields() -> serde_json::Value {
    json!([
        { "key": "create_project_enabled", "entity_type": "Opportunity", "label": "Create Project on Won", "field_type": "boolean", "options": [], "required": false, "show_in_list": false, "sort_order": 100, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": "When this opportunity is marked Won, automatically create a Project shell linked to it.", "placeholder": null }
    ])
}

fn work_package_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "work_package", "label": "Stage", "field_type": "select", "options": ["Planned", "Ready", "In Progress", "Blocked", "Complete"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "trade_scope", "entity_type": "work_package", "label": "Trade / Scope", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "budget", "entity_type": "work_package", "label": "Budget", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "start_date", "entity_type": "work_package", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "work_package", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Complete by the "Work package complete" business rule below.
        { "key": "completion_date", "entity_type": "work_package", "label": "Completion Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn change_order_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "change_order", "label": "Stage", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Rejected", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reason", "entity_type": "change_order", "label": "Reason", "field_type": "text", "options": [], "required": true, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "change_order", "label": "Requested Amount", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "requested_date", "entity_type": "change_order", "label": "Requested Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Both required-when-Approved by the "Change approval" business rule below.
        { "key": "approved_date", "entity_type": "change_order", "label": "Approved Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "approved_amount", "entity_type": "change_order", "label": "Approved Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn subcontract_assignment_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "subcontract_assignment", "label": "Stage", "field_type": "select", "options": ["Pending", "Confirmed", "Active", "Completed", "Cancelled"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "subcontract_assignment", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn inspection_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "inspection", "label": "Stage", "field_type": "select", "options": ["Planned", "Passed", "Failed", "Reinspection Required"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "inspection_type", "entity_type": "inspection", "label": "Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "due_date", "entity_type": "inspection", "label": "Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "completed_date", "entity_type": "inspection", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "notes", "entity_type": "inspection", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` and the
/// Opportunity-Won workflow's `create_record` action reference these
/// relationships by their position in this array.
fn construction_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "project", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Projects", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "work_package", "target_entity_type": "project", "relationship_type": "many_to_one", "forward_label": "Project", "reverse_label": "Work Packages", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "change_order", "target_entity_type": "project", "relationship_type": "many_to_one", "forward_label": "Project", "reverse_label": "Change Orders", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "subcontract_assignment", "target_entity_type": "work_package", "relationship_type": "many_to_one", "forward_label": "Work Package", "reverse_label": "Subcontract Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "subcontract_assignment", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Vendor", "reverse_label": "Subcontract Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "inspection", "target_entity_type": "project", "relationship_type": "many_to_one", "forward_label": "Project", "reverse_label": "Inspections", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "project", "target_entity_type": "Quote", "relationship_type": "many_to_one", "forward_label": "Quote", "reverse_label": "Projects", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 7 */ { "source_entity_type": "project", "target_entity_type": "Contract", "relationship_type": "many_to_one", "forward_label": "Contract", "reverse_label": "Projects", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 8 */ { "source_entity_type": "Invoice", "target_entity_type": "project", "relationship_type": "many_to_one", "forward_label": "Project", "reverse_label": "Invoices", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 9 */ { "source_entity_type": "project", "target_entity_type": "Opportunity", "relationship_type": "many_to_one", "forward_label": "Opportunity", "reverse_label": "Projects", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 }
    ])
}

fn construction_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "project",
            "name": "Close project",
            "description": "A closed project must record its actual end date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "actual_end_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "change_order",
            "name": "Change approval",
            "description": "An approved change order must record its approved amount and date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Approved" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "approved_amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "approved_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "work_package",
            "name": "Work package complete",
            "description": "A completed work package must record its completion date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Complete" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completion_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Vendor integrity" (spec: a Subcontract Assignment's vendor Company
/// must be marked Vendor/active) is left out - it reads a *related*
/// Company record's own field from a Subcontract Assignment condition,
/// the same cross-record gap ruled out for Field Service's Asset/Site
/// integrity and Property Management's occupancy conflict.
fn construction_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "Opportunity",
            "name": "Opportunity won creates project",
            "description": "Winning an opportunity with 'Create Project' enabled spins up a Project shell linked to it.",
            "trigger_type": "status_changed",
            "trigger_status": "Won",
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "create_project_enabled", "operator": "equals", "value": "true" }
            ],
            "actions": [
                { "action_type": "create_record", "params_json": "{\"entity_type\":\"project\",\"relationship_ref\":9,\"name_template\":null}" }
            ]
        },
        {
            "entity_type": "change_order",
            "name": "Change order approved",
            "description": "Approving a change order notifies admins to update the project's approved change value - see this module's own doc comment on why that update isn't automated.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Approved" }
            ],
            "actions": [
                { "action_type": "add_notification", "params_json": "{\"message\":\"A change order was approved - update the project's approved change value.\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "inspection",
            "name": "Inspection failed",
            "description": "A failed inspection opens a corrective task and lets admins know.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Failed" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Address failed inspection\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" },
                { "action_type": "add_notification", "params_json": "{\"message\":\"An inspection failed\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "project",
            "name": "Project close",
            "description": "Closing a project opens a final billing review task - see this module's own doc comment on why the spec's 'close outstanding auto-close tasks' clause isn't automated.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Final billing review\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        }
    ])
}
