//! Reference industry packages - real manifests written against
//! `models::industry_package`/`industry_package_service`, kept in Rust
//! (not shipped as loose files) so they're compiled, testable, and
//! versioned with the engine they target. The dev spec ("Top 10 Industry
//! Data Models & Packaged Business Apps") sequences Field Service,
//! Property Management, Construction & Contractors, Professional
//! Services, Dental/Clinic Practice Administration, Recruitment &
//! Staffing, Real Estate Brokerage, Legal Practice, Nonprofit &
//! Association and Auto Repair / Service Garage first; this module ships
//! all ten, proving the foundation against real content rather than only
//! the synthetic manifests `industry_data_model.rs`'s tests use.
//!
//! An eleventh package, Policy Administration & Claims Management
//! (`policy_admin_claims_manifest_json`), was added afterward on direct
//! product request - insurance carriers and MGAs weren't in the original
//! ten-vertical spec. It follows the exact same manifest shape and the
//! same engine-limitation list below; see its own doc comment for the
//! real-world policy admin/claims systems (Guidewire PolicyCenter/
//! ClaimCenter, Duck Creek, and the ACORD data standards) its object
//! model and vocabulary are checked against.
//!
//! `field_service_manifest_json` was later deepened past its original
//! ten-vertical scope, on direct product request to check it against how
//! real field service management (FSM) software actually shapes this
//! data - ServiceTitan and Salesforce Field Service Lightning - rather
//! than stopping at the spec's original Work Order/Appointment/Asset
//! core. See that function's own doc comment for what was added and why.
//!
//! A twelfth package, Lanesra Industry Foundation
//! (`lanesra_industry_foundation_manifest_json`), is different in kind
//! from the eleven above it: it's shared cross-industry plumbing (Party,
//! Party Role, Location, Asset, Agreement, ...) other packages can
//! optionally *depend on*, not its own business vertical. It's the first
//! package in this module to use `IndustryPackageManifest::dependencies`
//! for real - `industry_package_service::validate` already enforces a
//! required dependency (installed, active, version-satisfying) before
//! `install`/`apply_update` runs, and a later package's own
//! objects/fields/relationships can already target an earlier package's
//! object key with no package-boundary check at all
//! (`custom_object_service::is_valid_dynamic_entity_type` is a plain
//! workspace-wide key lookup). Confirmed by reading that validation code
//! directly, not assumed - see its own doc comment for exactly what it
//! ships, what it deliberately leaves out of the source spec's object
//! list, and why.
//!
//! Two packages both needing a "Project"-shaped object (Construction &
//! Contractors and Professional Services) is also the first real test of
//! packages coexisting in one workspace: a custom object's key is
//! workspace-wide, so Professional Services' equivalent object is named
//! `engagement`, not `project` - see `professional_services_manifest_json`'s
//! own doc comment for the same consideration applied to a custom field
//! it adds to the shared built-in Opportunity entity.
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
//!   maintenance workflow, Property Management's lease-renewal/
//!   document-expiry ones, and Professional Services' "Milestone due
//!   soon" reminder.
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
//! - No time-of-day or timestamp field type - `CUSTOM_FIELD_TYPES` is
//!   `text`/`number`/`date`/`boolean`/`select`, nothing finer-grained than
//!   a whole day. Practice Administration's Appointment "date/time" and
//!   "completion timestamp" are both approximated with a plain date field
//!   (plus a free-text time-of-day field for the former) rather than a
//!   real time value - see `practice_admin_manifest_json`'s own doc
//!   comment.
//! - A business rule requiring that a relationship link *exists*, not
//!   just that a field has a value - the `require` action only ever
//!   targets a field key. Practice Administration's "Appointment must
//!   have a Patient and Provider" and "a Patient Profile must link to
//!   exactly one Contact" are both left unenforced by a rule for this
//!   reason (the relationship definitions' own `is_required` flag still
//!   records the intent).
//! - Overlap/conflict detection across a set of sibling records (Practice
//!   Administration's "block an overlapping Provider appointment") - this
//!   needs both the cross-record read above *and* scanning every other
//!   record sharing the same relationship, neither of which the condition
//!   engine can do; it's not simply a bigger instance of the cross-record
//!   gap, since even a single related record's field wouldn't be enough
//!   to answer it.
//! - A condition compared against "now"/"today", not just a literal or
//!   another field - every operator's right-hand side is either a fixed
//!   value or another field on the same record, never the current
//!   date/time at evaluation time. Recruitment & Staffing's "Interview
//!   scheduling must be in the future unless already completed" is left
//!   out for this reason - see `recruitment_business_rules`'s own doc
//!   comment.
//! - `create_record`'s `name_template` is a fixed literal string (or, if
//!   omitted, "Related to {triggering record's name}"), not a per-record
//!   template that can interpolate the triggering record's own field
//!   values into the new record. A workflow that both creates a linked
//!   record *and* copies field values onto it (Auto Repair / Service
//!   Garage's "Appointment check-in ... copy customer/vehicle [context]")
//!   can only do the create-and-link half; see `auto_service_workflows`'s
//!   own doc comment.
//! - A self-referential relationship - an object related to another
//!   record of its own same type (a parent/child hierarchy: an
//!   Organization Unit's parent unit, a WBS Element's parent WBS, an
//!   Asset's parent Asset). `relationship_service::validate_shape`
//!   rejects `source_entity_type == target_entity_type` outright ("A
//!   relationship must connect two different object types") - confirmed
//!   by reading that function, not assumed. Every hierarchy in this
//!   module is either flattened to one level (no sub-assemblies) or, where
//!   a parent link is genuinely part of the spec (Lanesra Industry
//!   Foundation's own Organization Unit), recorded as a plain unenforced
//!   text field instead of a real link; see
//!   `lanesra_industry_foundation_manifest_json`'s own doc comment. Field
//!   Service's own "callback job points back at the original Work Order"
//!   hits the same gap and is modeled as a plain boolean flag
//!   (`is_callback`) instead, with no link to which earlier Work Order it
//!   was a callback for; see `field_service_fields`'s own doc comment.
//! - A generic polymorphic relationship target - "this record relates to
//!   *any* other record, of whatever type" (a document attached to a
//!   policy or a claim or a matter; an external identifier for whatever
//!   object it maps to). A `RelationshipDefinitionInput`'s
//!   `target_entity_type` is always one fixed type, chosen at manifest-
//!   authoring time, never resolved per-record at runtime. Lanesra
//!   Industry Foundation's Document Record, External Identifier,
//!   Interaction, Financial Transaction and Data Quality Issue objects
//!   all want exactly this and don't get it - see that package's own doc
//!   comment for how each is degraded to a plain, unenforced pair of text
//!   fields instead.

use serde_json::json;

/// `lanesra.field_service` v1.0.0 - see this module's own doc comment for
/// what's included and what's deliberately left out. Returns the raw
/// manifest JSON text, the same shape `ImportPackageInput::manifest_json`
/// expects, so a caller can either import it as-is or show it to an
/// admin for review first (the Admin -> App Catalog screen's "Load
/// starter package" button does the latter, matching the spec's own
/// Review -> Validate -> Install flow instead of a silent one-click
/// install).
///
/// Deepened past the original ten-vertical spec's Work Order/Appointment/
/// Asset/Warranty Claim core, checked against how ServiceTitan and
/// Salesforce Field Service Lightning actually shape the rest of a real
/// FSM business:
/// - `service_estimate` / `service_estimate_line` - both tools separate a
///   pre-authorization estimate (options presented to the customer, with
///   its own approve/decline lifecycle) from the eventual job; a Work
///   Order can now optionally point back at the estimate it was created
///   from, the same "document that precedes and can become another
///   document" shape Field Service's own `work_order_line` already uses
///   for its own object.
/// - `maintenance_plan` - ServiceTitan's "Membership"/recurring service
///   agreement and Salesforce's "Maintenance Plan": a contracted, ongoing
///   revenue relationship distinct from a one-off Work Order, which an
///   Asset can optionally be covered by and a Work Order can optionally
///   be generated under. Recurring Work Order *generation* on a schedule
///   would need a `date_reached` trigger on a custom object's own date
///   field - this engine doesn't have one (see this module's own doc
///   comment) - so which visits happen when stays a dispatcher decision;
///   the plan only records the agreement's own terms and which asset/
///   customer it covers.
/// - `time_entry` - per-technician labor hours against a Work Order, the
///   basic unit both tools use for job costing and payroll, entirely
///   absent from the original model.
/// - `work_order.is_callback` - both tools track whether a job is a
///   return visit for unresolved prior work, a real quality metric; see
///   this module's own doc comment for why it isn't linked back to the
///   original Work Order.
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
            { "key": "service_territory", "singular_label": "Service Territory", "plural_label": "Service Territories", "icon": "🗺", "prefix": "TER", "digits": 3 },
            { "key": "warranty_claim", "singular_label": "Warranty Claim", "plural_label": "Warranty Claims", "icon": "🧾", "prefix": "WC", "digits": 5 },
            { "key": "service_estimate", "singular_label": "Estimate", "plural_label": "Estimates", "icon": "📝", "prefix": "EST", "digits": 5 },
            { "key": "service_estimate_line", "singular_label": "Estimate Line", "plural_label": "Estimate Lines", "icon": "📄", "prefix": "ESL", "digits": 5 },
            { "key": "maintenance_plan", "singular_label": "Maintenance Plan", "plural_label": "Maintenance Plans", "icon": "🔁", "prefix": "MPL", "digits": 4 },
            { "key": "field_time_entry", "singular_label": "Time Entry", "plural_label": "Time Entries", "icon": "⏱", "prefix": "FTE", "digits": 6 }
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
                                    "fields": ["description", "stage", "priority", "requested_date", "resolution", "completion_date", "is_callback"]
                                }
                            ],
                            // Indices into `relationships` below: Asset (4), Appointments (6), Lines (5) -
                            // resolved to real relationship keys by the installer.
                            "related": ["4", "6", "5"]
                        }
                    ]
                },
                "publish": true
            },
            {
                "entity_type": "service_estimate",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "overview",
                            "title": "Overview",
                            "sections": [
                                { "id": "details", "title": "Details", "columns": 2, "fields": ["estimate_status", "subtotal_amount", "tax_amount", "total_amount", "presented_date", "approved_date", "decline_reason"] }
                            ],
                            // Indices into `relationships` below: Estimate Lines (15), Work Orders Created (17).
                            "related": ["15", "17"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Work Orders by Stage", "entity_type": "work_order", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null },
            { "name": "Estimates by Status", "entity_type": "service_estimate", "group_by_source": "custom", "group_by_field": "estimate_status", "aggregate": "count", "sum_field_key": null },
            { "name": "Maintenance Plans by Status", "entity_type": "maintenance_plan", "group_by_source": "custom", "group_by_field": "plan_status", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Field Service Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } },
                { "kind": "chart", "config": { "report_ref": 1, "chart_type": "bar" } },
                { "kind": "chart", "config": { "report_ref": 2, "chart_type": "bar" } }
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
                "work_type", "resource_profile", "skill", "service_territory", "warranty_claim",
                "service_estimate", "service_estimate_line", "maintenance_plan", "field_time_entry", "Task"
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
        warranty_claim_fields(),
        service_estimate_fields(),
        service_estimate_line_fields(),
        maintenance_plan_fields(),
        field_time_entry_fields(),
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
        { "key": "completion_date", "entity_type": "work_order", "label": "Completion Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // See this module's own doc comment: a self-referential
        // relationship back to the original Work Order isn't possible
        // (`source_entity_type == target_entity_type` is rejected), so a
        // callback is flagged here rather than linked to what it was a
        // callback for.
        { "key": "is_callback", "entity_type": "work_order", "label": "Callback", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "false", "is_unique": false, "help_text": "A return visit for unresolved prior work, not a new issue.", "placeholder": null }
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

fn warranty_claim_fields() -> serde_json::Value {
    json!([
        { "key": "claim_status", "entity_type": "warranty_claim", "label": "Status", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Denied", "Reimbursed"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount_requested", "entity_type": "warranty_claim", "label": "Amount Requested", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Reimbursed by the "Claim reimbursement amount" business rule below.
        { "key": "amount_approved", "entity_type": "warranty_claim", "label": "Amount Approved", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "submitted_date", "entity_type": "warranty_claim", "label": "Submitted Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-resolved (Approved/Denied/Reimbursed) by the "Claim resolution notes" business rule below.
        { "key": "resolution_notes", "entity_type": "warranty_claim", "label": "Resolution Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_estimate_fields() -> serde_json::Value {
    json!([
        { "key": "estimate_status", "entity_type": "service_estimate", "label": "Status", "field_type": "select", "options": ["Draft", "Presented", "Approved", "Declined", "Expired"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "subtotal_amount", "entity_type": "service_estimate", "label": "Subtotal", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "tax_amount", "entity_type": "service_estimate", "label": "Tax", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Approved by the "Estimate approval requires totals" business rule below.
        { "key": "total_amount", "entity_type": "service_estimate", "label": "Total", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "presented_date", "entity_type": "service_estimate", "label": "Presented Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Approved by the "Estimate approval requires totals" business rule below.
        { "key": "approved_date", "entity_type": "service_estimate", "label": "Approved Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Declined by the "Estimate decline requires reason" business rule below.
        { "key": "decline_reason", "entity_type": "service_estimate", "label": "Decline Reason", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_estimate_line_fields() -> serde_json::Value {
    json!([
        { "key": "line_description", "entity_type": "service_estimate_line", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "quantity", "entity_type": "service_estimate_line", "label": "Quantity", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "1", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "unit_price", "entity_type": "service_estimate_line", "label": "Unit Price", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // A plain field the estimator enters, not a rollup - this engine
        // has no aggregate/computed field type (see this module's own doc
        // comment; `work_order_line` has the identical gap already).
        { "key": "line_total", "entity_type": "service_estimate_line", "label": "Line Total", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn maintenance_plan_fields() -> serde_json::Value {
    json!([
        { "key": "plan_status", "entity_type": "maintenance_plan", "label": "Status", "field_type": "select", "options": ["Active", "Expired", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "plan_type", "entity_type": "maintenance_plan", "label": "Plan Type", "field_type": "select", "options": ["Annual Tune-Up", "Quarterly Filter Change", "Custom"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "start_date", "entity_type": "maintenance_plan", "label": "Start Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-non-renewing by the "Non-renewing plan requires an end date" business rule below.
        { "key": "end_date", "entity_type": "maintenance_plan", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "visit_frequency", "entity_type": "maintenance_plan", "label": "Visit Frequency", "field_type": "select", "options": ["Monthly", "Quarterly", "Semi-Annual", "Annual"], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "included_visits_per_term", "entity_type": "maintenance_plan", "label": "Included Visits per Term", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Manually incremented by a dispatcher - `update_related_record`
        // only ever sets a literal or a copied value, it never reads and
        // increments a target's current value (see this module's own doc
        // comment), so there's no automated way to bump this when a
        // covered Work Order completes.
        { "key": "visits_used", "entity_type": "maintenance_plan", "label": "Visits Used This Term", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "0", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "auto_renew", "entity_type": "maintenance_plan", "label": "Auto-Renew", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "annual_value", "entity_type": "maintenance_plan", "label": "Annual Value", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 8, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

// Named field_time_entry, not time_entry - Professional Services already
// claims that object key for its own billable-hours object (and Legal
// Practice's own equivalent is `matter_time_entry` for the identical
// reason); see this module's own doc comment on workspace-wide object
// key uniqueness.
fn field_time_entry_fields() -> serde_json::Value {
    json!([
        { "key": "entry_date", "entity_type": "field_time_entry", "label": "Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "hours", "entity_type": "field_time_entry", "label": "Hours", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": "24", "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Overtime requires justification" business rule below.
        { "key": "labor_type", "entity_type": "field_time_entry", "label": "Labor Type", "field_type": "select", "options": ["Standard", "Overtime", "Travel"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Standard", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "field_time_entry", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Overtime by the "Overtime requires justification" business rule below.
        { "key": "notes", "entity_type": "field_time_entry", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: `screen_layouts[0].draft`'s `related`
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
        /* 10 */ { "source_entity_type": "work_order_line", "target_entity_type": "Product", "relationship_type": "many_to_one", "forward_label": "Product / Service", "reverse_label": "Work Order Lines", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 11 */ { "source_entity_type": "warranty_claim", "target_entity_type": "asset", "relationship_type": "many_to_one", "forward_label": "Asset", "reverse_label": "Warranty Claims", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 12 */ { "source_entity_type": "warranty_claim", "target_entity_type": "work_order", "relationship_type": "many_to_one", "forward_label": "Work Order", "reverse_label": "Warranty Claims", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 13 */ { "source_entity_type": "service_estimate", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Estimates", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 14 */ { "source_entity_type": "service_estimate", "target_entity_type": "service_site", "relationship_type": "many_to_one", "forward_label": "Service Site", "reverse_label": "Estimates", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 15 */ { "source_entity_type": "service_estimate_line", "target_entity_type": "service_estimate", "relationship_type": "many_to_one", "forward_label": "Estimate", "reverse_label": "Lines", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 16 */ { "source_entity_type": "service_estimate_line", "target_entity_type": "Product", "relationship_type": "many_to_one", "forward_label": "Product / Service", "reverse_label": "Estimate Lines", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        // work_order's own sort_order sequence continues from index 9 above (Contract, sort_order 3).
        /* 17 */ { "source_entity_type": "work_order", "target_entity_type": "service_estimate", "relationship_type": "many_to_one", "forward_label": "Originating Estimate", "reverse_label": "Work Orders Created", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 18 */ { "source_entity_type": "maintenance_plan", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Maintenance Plans", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        // asset's own sort_order sequence continues from index 1 above (Service Site, sort_order 0).
        /* 19 */ { "source_entity_type": "asset", "target_entity_type": "maintenance_plan", "relationship_type": "many_to_one", "forward_label": "Covered By Plan", "reverse_label": "Covered Assets", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 20 */ { "source_entity_type": "work_order", "target_entity_type": "maintenance_plan", "relationship_type": "many_to_one", "forward_label": "Maintenance Plan", "reverse_label": "Work Orders", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 5 },
        /* 21 */ { "source_entity_type": "field_time_entry", "target_entity_type": "work_order", "relationship_type": "many_to_one", "forward_label": "Work Order", "reverse_label": "Time Entries", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 22 */ { "source_entity_type": "field_time_entry", "target_entity_type": "resource_profile", "relationship_type": "many_to_one", "forward_label": "Technician", "reverse_label": "Time Entries", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 }
    ])
}

/// The original spec's rules are limited to the two a same-record
/// condition/action engine can actually express - "Asset/site integrity"
/// and "Warranty warning" both need to read a *related* record's own
/// fields, which conditions can't do; see this module's own doc comment.
/// The four below it were added with the Estimate/Maintenance Plan/Time
/// Entry deepening (see `field_service_manifest_json`'s own doc comment)
/// and hold to the same limit.
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
        },
        {
            "entity_type": "warranty_claim",
            "name": "Claim resolution notes",
            "description": "A resolved warranty claim must record its resolution notes.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "in_list", "value": "Approved|Denied|Reimbursed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "resolution_notes", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "warranty_claim",
            "name": "Claim reimbursement amount",
            "description": "A reimbursed warranty claim must record its approved amount.",
            "match_type": "all",
            "priority": 1,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Reimbursed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "amount_approved", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "service_estimate",
            "name": "Estimate approval requires totals",
            "description": "An approved estimate must record its total amount and approval date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "estimate_status", "operator": "equals", "value": "Approved" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "total_amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "approved_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "service_estimate",
            "name": "Estimate decline requires reason",
            "description": "A declined estimate must record why.",
            "match_type": "all",
            "priority": 1,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "estimate_status", "operator": "equals", "value": "Declined" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "decline_reason", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "maintenance_plan",
            "name": "Non-renewing plan requires an end date",
            "description": "A plan that won't auto-renew must record when it ends.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "auto_renew", "operator": "equals", "value": "false" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "end_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "field_time_entry",
            "name": "Overtime requires justification",
            "description": "An overtime time entry must record why the extra hours were needed.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "labor_type", "operator": "equals", "value": "Overtime" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "notes", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// Only two of the original spec's five workflows. Left out, beyond the
/// two already noted in this module's own doc comment:
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
///
/// The three after "Warranty claim submitted" were added with the
/// Estimate/Maintenance Plan deepening (see `field_service_manifest_json`'s
/// own doc comment). "Plan expiring soon" would need the same
/// unsupported `date_reached` trigger as "Preventive maintenance" above,
/// so it's left out for the identical reason.
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
        },
        {
            "entity_type": "warranty_claim",
            "name": "Warranty claim submitted",
            "description": "Submitting a warranty claim opens a review task and lets admins know.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "claim_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Submitted" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Review warranty claim\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" },
                { "action_type": "add_notification", "params_json": "{\"message\":\"A warranty claim was submitted\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "service_estimate",
            "name": "Estimate presented",
            "description": "Presenting an estimate to the customer opens a follow-up task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "estimate_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "estimate_status", "operator": "equals", "value": "Presented" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up with customer on estimate\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "service_estimate",
            "name": "Estimate approved",
            "description": "An approved estimate needs its work order scheduled and lets the service manager know.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "estimate_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "estimate_status", "operator": "equals", "value": "Approved" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Schedule a work order from this approved estimate\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" },
                { "action_type": "add_notification", "params_json": "{\"message\":\"An estimate was approved\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "maintenance_plan",
            "name": "Plan cancelled",
            "description": "Cancelling a maintenance plan opens a task to confirm the cancellation and stop billing.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "plan_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "plan_status", "operator": "equals", "value": "Cancelled" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Confirm cancellation and stop billing\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
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
            { "key": "property_document", "singular_label": "Property Document", "plural_label": "Property Documents", "icon": "📄", "prefix": "DOC", "digits": 5 },
            { "key": "unit_showing", "singular_label": "Unit Showing", "plural_label": "Unit Showings", "icon": "👁", "prefix": "SHW", "digits": 5 }
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
                "maintenance_request", "vendor_assignment", "property_document", "unit_showing", "Task"
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
        unit_showing_fields(),
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

fn unit_showing_fields() -> serde_json::Value {
    json!([
        { "key": "showing_stage", "entity_type": "unit_showing", "label": "Stage", "field_type": "select", "options": ["Scheduled", "Completed", "Cancelled", "No Show"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Scheduled", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "scheduled_date", "entity_type": "unit_showing", "label": "Scheduled Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed by the "Showing completion" business rule below.
        { "key": "interest_level", "entity_type": "unit_showing", "label": "Interest Level", "field_type": "select", "options": ["Low", "Medium", "High"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "showing_notes", "entity_type": "unit_showing", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
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
        /* 10 */ { "source_entity_type": "property_document", "target_entity_type": "property", "relationship_type": "many_to_one", "forward_label": "Property", "reverse_label": "Documents", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 3 },
        /* 11 */ { "source_entity_type": "unit_showing", "target_entity_type": "unit", "relationship_type": "many_to_one", "forward_label": "Unit", "reverse_label": "Showings", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 12 */ { "source_entity_type": "unit_showing", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Prospect", "reverse_label": "Unit Showings", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 }
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
        },
        {
            "entity_type": "unit_showing",
            "name": "Showing completion",
            "description": "A completed showing must record the prospect's interest level.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "showing_stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "interest_level", "target_field_source": "custom", "action_value": null, "message": null }
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
        },
        {
            "entity_type": "unit_showing",
            "name": "High-interest showing follow-up",
            "description": "A completed showing where the prospect showed high interest gets an agent follow-up task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "showing_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "showing_stage", "operator": "equals", "value": "Completed" },
                { "field_source": "custom", "field_key": "interest_level", "operator": "equals", "value": "High" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up with interested prospect\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
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
            { "key": "inspection", "singular_label": "Inspection", "plural_label": "Inspections", "icon": "🔎", "prefix": "INSP", "digits": 4 },
            { "key": "punch_list_item", "singular_label": "Punch List Item", "plural_label": "Punch List Items", "icon": "📋", "prefix": "PLI", "digits": 4 }
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
                "project", "work_package", "change_order", "subcontract_assignment", "inspection", "punch_list_item", "Task"
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
        punch_list_item_fields(),
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

fn punch_list_item_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "punch_list_item", "label": "Stage", "field_type": "select", "options": ["Open", "In Progress", "Resolved", "Verified", "Won't Fix"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Open", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description", "entity_type": "punch_list_item", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "location", "entity_type": "punch_list_item", "label": "Location", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Where on the site this item was found.", "placeholder": null },
        { "key": "due_date", "entity_type": "punch_list_item", "label": "Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Resolved/Verified by the "Punch item resolution" business rule below.
        { "key": "resolved_date", "entity_type": "punch_list_item", "label": "Resolved Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
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
        /* 9 */ { "source_entity_type": "project", "target_entity_type": "Opportunity", "relationship_type": "many_to_one", "forward_label": "Opportunity", "reverse_label": "Projects", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 10 */ { "source_entity_type": "punch_list_item", "target_entity_type": "project", "relationship_type": "many_to_one", "forward_label": "Project", "reverse_label": "Punch List Items", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 5 }
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
        },
        {
            "entity_type": "punch_list_item",
            "name": "Punch item resolution",
            "description": "A resolved or verified punch list item must record its resolved date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "in_list", "value": "Resolved|Verified" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "resolved_date", "target_field_source": "custom", "action_value": null, "message": null }
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
        },
        {
            "entity_type": "punch_list_item",
            "name": "Punch item created",
            "description": "Logging a new punch list item opens a task to address it.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Address punch list item\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.professional_services ---------------------------------------

/// `lanesra.professional_services` v1.0.0 - the fourth package, sequenced
/// right after Construction & Contractors per the dev spec. See this
/// module's own doc comment for what's included and deliberately left
/// out.
///
/// Named `engagement`, not `project` - Construction & Contractors already
/// claims the `project` custom object key, and a custom object's key is
/// workspace-wide (`custom_object_service::create_with_key` hard-fails on
/// a collision), not per-package. A workspace should be free to install
/// both packages side by side. The same consideration applies to the
/// custom field this package adds to the shared built-in Opportunity
/// entity: it's `create_engagement_enabled`, not Construction's
/// `create_project_enabled`, for the identical reason (custom field keys
/// are unique per `(entity_type, key)`, and Opportunity is the same
/// built-in entity both packages attach to).
///
/// "Resource Assignment: Project <-> User/Resource" doesn't use a
/// relationship the way every other object connection here does - `User`
/// isn't one of the nine relationship-capable entity types
/// (`entity_registry::ALL`), since a user account isn't a CRM record.
/// Resource Assignment's own built-in owner field (present on every
/// custom object) stands in for "which user this assignment is for"
/// instead, the same field a Task or Company already uses to mean
/// "assigned to".
pub fn professional_services_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.professional_services",
        "name": "Professional Services",
        "industry": "Professional Services",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "engagement", "singular_label": "Engagement", "plural_label": "Engagements", "icon": "💼", "prefix": "ENG", "digits": 4 },
            { "key": "milestone", "singular_label": "Milestone", "plural_label": "Milestones", "icon": "🚩", "prefix": "MS", "digits": 4 },
            { "key": "resource_assignment", "singular_label": "Resource Assignment", "plural_label": "Resource Assignments", "icon": "🧑‍💻", "prefix": "RA", "digits": 4 },
            { "key": "time_entry", "singular_label": "Time Entry", "plural_label": "Time Entries", "icon": "⏱", "prefix": "TE", "digits": 5 },
            { "key": "expense", "singular_label": "Expense", "plural_label": "Expenses", "icon": "💳", "prefix": "EXP", "digits": 5 },
            { "key": "deliverable", "singular_label": "Deliverable", "plural_label": "Deliverables", "icon": "📦", "prefix": "DLV", "digits": 4 },
            { "key": "change_request", "singular_label": "Change Request", "plural_label": "Change Requests", "icon": "🔄", "prefix": "CR", "digits": 4 }
        ],
        "fields": professional_services_fields(),
        "relationships": professional_services_relationships(),
        "business_rules": professional_services_business_rules(),
        "workflows": professional_services_workflows(),
        "screen_layouts": [
            {
                "entity_type": "engagement",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["stage", "billing_model", "contract_value", "start_date", "end_date", "actual_end_date"] }
                            ],
                            // Indices into `relationships` below: Milestones (1), Resource Assignments (2), Time Entries (3), Expenses (5), Deliverables (6).
                            "related": ["1", "2", "3", "5", "6"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Engagements by Stage", "entity_type": "engagement", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Professional Services Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Professional Services",
            "icon": "💼",
            "description": "Engagements, milestones, resourcing, time and expenses for consulting, IT services, agencies and other billable professional-services firms.",
            "object_keys": [
                "engagement", "milestone", "resource_assignment", "time_entry", "expense", "deliverable", "change_request", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names (PSA Admin, Project Manager, Consultant, Practice Manager) onto
            // this build's actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "editor" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object exists here either - every object is a
        // live delivery/billing record.
        "seed_data": []
    })
    .to_string()
}

fn professional_services_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        engagement_fields(),
        professional_services_opportunity_fields(),
        milestone_fields(),
        resource_assignment_fields(),
        time_entry_fields(),
        expense_fields(),
        deliverable_fields(),
        change_request_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn engagement_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "engagement", "label": "Stage", "field_type": "select", "options": ["Planned", "Active", "On Hold", "Complete", "Closed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billing_model", "entity_type": "engagement", "label": "Billing Model", "field_type": "select", "options": ["Time & Materials", "Fixed Fee", "Retainer"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Time & Materials", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "contract_value", "entity_type": "engagement", "label": "Contract Value", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "start_date", "entity_type": "engagement", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "engagement", "label": "Planned End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Complete by the "Engagement completion" business rule below.
        { "key": "actual_end_date", "entity_type": "engagement", "label": "Actual End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// See this module's own doc comment on why this key is
/// `create_engagement_enabled`, not Construction & Contractors'
/// `create_project_enabled`.
fn professional_services_opportunity_fields() -> serde_json::Value {
    json!([
        { "key": "create_engagement_enabled", "entity_type": "Opportunity", "label": "Create Engagement on Won", "field_type": "boolean", "options": [], "required": false, "show_in_list": false, "sort_order": 101, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": "When this opportunity is marked Won, automatically create an Engagement draft linked to it.", "placeholder": null }
    ])
}

fn milestone_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "milestone", "label": "Stage", "field_type": "select", "options": ["Not Started", "In Progress", "At Risk", "Complete"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Not Started", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "due_date", "entity_type": "milestone", "label": "Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Reminder automation isn't available for a custom object's own date fields - see this module's own doc comment.", "placeholder": null },
        { "key": "amount", "entity_type": "milestone", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Complete by the "Milestone completion" business rule below.
        { "key": "completed_date", "entity_type": "milestone", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn resource_assignment_fields() -> serde_json::Value {
    json!([
        { "key": "role", "entity_type": "resource_assignment", "label": "Role", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "allocation_percent", "entity_type": "resource_assignment", "label": "Allocation %", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": "100", "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "start_date", "entity_type": "resource_assignment", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "resource_assignment", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "bill_rate", "entity_type": "resource_assignment", "label": "Bill Rate", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn time_entry_fields() -> serde_json::Value {
    json!([
        { "key": "date", "entity_type": "time_entry", "label": "Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-greater-than-zero-when-Submitted by the "Time submission" business rule below.
        { "key": "hours", "entity_type": "time_entry", "label": "Hours", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "time_entry", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "time_entry", "label": "Stage", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Rejected"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        // Entered manually rather than snapshotted from the linked Resource Assignment's own
        // bill_rate at creation time - that snapshot would need to read a *related* record's
        // field, the same cross-record gap this module's own doc comment already covers.
        { "key": "bill_rate", "entity_type": "time_entry", "label": "Bill Rate", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": "Not auto-copied from the resource's assignment rate - enter it directly.", "placeholder": null },
        // Written by the "Time approved" workflow below.
        { "key": "billing_status", "entity_type": "time_entry", "label": "Billing Status", "field_type": "select", "options": ["Not Billed", "Eligible", "Billed"], "required": false, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Not Billed", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn expense_fields() -> serde_json::Value {
    json!([
        { "key": "date", "entity_type": "expense", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "expense", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "category", "entity_type": "expense", "label": "Category", "field_type": "select", "options": ["Travel", "Meals", "Materials", "Software", "Other"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "expense", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        // category/amount/date are required-when-Submitted by the "Expense
        // submission" business rule below, not required at creation.
        { "key": "stage", "entity_type": "expense", "label": "Stage", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Rejected"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn deliverable_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "deliverable", "label": "Stage", "field_type": "select", "options": ["Not Started", "In Progress", "Complete"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Not Started", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "due_date", "entity_type": "deliverable", "label": "Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "acceptance", "entity_type": "deliverable", "label": "Acceptance", "field_type": "select", "options": ["Pending", "Accepted", "Rejected"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn change_request_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "change_request", "label": "Stage", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Rejected", "Implemented"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description", "entity_type": "change_request", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "requested_date", "entity_type": "change_request", "label": "Requested Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "impact_amount", "entity_type": "change_request", "label": "Contract Value Impact", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": "Positive or negative change to the engagement's contract value.", "placeholder": null },
        // Required-when-Approved/Implemented by the "Change request approval" business rule below.
        { "key": "approved_date", "entity_type": "change_request", "label": "Approved Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` and the
/// Opportunity-Won workflow's `create_record` action reference these
/// relationships by their position in this array.
fn professional_services_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "engagement", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Engagements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "milestone", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Milestones", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "resource_assignment", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Resource Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "time_entry", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Time Entries", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 4 */ { "source_entity_type": "time_entry", "target_entity_type": "milestone", "relationship_type": "many_to_one", "forward_label": "Milestone", "reverse_label": "Time Entries", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "expense", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Expenses", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 6 */ { "source_entity_type": "deliverable", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Deliverables", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 7 */ { "source_entity_type": "engagement", "target_entity_type": "Opportunity", "relationship_type": "many_to_one", "forward_label": "Opportunity", "reverse_label": "Engagements", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 8 */ { "source_entity_type": "engagement", "target_entity_type": "Contract", "relationship_type": "many_to_one", "forward_label": "Contract", "reverse_label": "Engagements", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 9 */ { "source_entity_type": "Invoice", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Invoices", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 10 */ { "source_entity_type": "change_request", "target_entity_type": "engagement", "relationship_type": "many_to_one", "forward_label": "Engagement", "reverse_label": "Change Requests", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 5 }
    ])
}

/// "Time submission" only enforces `hours > 0` (via `less_than "0.01"` on
/// the violating side, since the condition engine has no `<=` operator)
/// when a time entry moves to Submitted - the spec's other half, "AND
/// Project Active", would need this rule to read the *linked* engagement's
/// own stage field, the same cross-record gap this module's own doc
/// comment already covers.
fn professional_services_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "engagement",
            "name": "Engagement completion",
            "description": "A complete engagement must record its actual end date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Complete" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "actual_end_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "time_entry",
            "name": "Time submission",
            "description": "A submitted time entry must record more than zero hours.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Submitted" },
                { "field_source": "custom", "field_key": "hours", "operator": "less_than", "value": "0.01" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Enter more than zero hours before submitting a time entry." }
            ]
        },
        {
            "entity_type": "expense",
            "name": "Expense submission",
            "description": "A submitted expense must record its category, amount and date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Submitted" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "category", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "milestone",
            "name": "Milestone completion",
            "description": "A completed milestone must record its completed date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Complete" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "change_request",
            "name": "Change request approval",
            "description": "An approved or implemented change request must record its approved date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "in_list", "value": "Approved|Implemented" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "approved_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Milestone due soon" (spec: due in 7 days AND not Complete creates a
/// reminder task) is left out - it needs a `date_reached`-style trigger on
/// a custom object's own date field, which this module's own doc comment
/// already rules out.
fn professional_services_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "Opportunity",
            "name": "Opportunity won creates engagement",
            "description": "Winning an opportunity with 'Create Engagement' enabled spins up an Engagement draft linked to it.",
            "trigger_type": "status_changed",
            "trigger_status": "Won",
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "create_engagement_enabled", "operator": "equals", "value": "true" }
            ],
            "actions": [
                { "action_type": "create_record", "params_json": "{\"entity_type\":\"engagement\",\"relationship_ref\":7,\"name_template\":null}" }
            ]
        },
        {
            "entity_type": "time_entry",
            "name": "Time approved",
            "description": "Approved time is marked eligible for the billing pool.",
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
                { "action_type": "update_field", "params_json": "{\"target_field_key\":\"billing_status\",\"target_field_source\":\"custom\",\"value\":\"Eligible\",\"copy_from_field_key\":null}" }
            ]
        },
        {
            "entity_type": "engagement",
            "name": "Engagement complete",
            "description": "Completing an engagement opens a closure/review task and an invoice-preparation task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Complete" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Engagement closure and review\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" },
                { "action_type": "create_task", "params_json": "{\"title\":\"Prepare final invoice\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "change_request",
            "name": "Change request submitted",
            "description": "Submitting a change request opens a review task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Submitted" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Review change request\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.practice_admin -----------------------------------------------

/// `lanesra.practice_admin` v1.0.0 - the fifth package, sequenced right
/// after Professional Services per the dev spec. Explicit scope
/// boundary, straight from the spec: administrative/operational practice
/// management only - not an EHR/EMR, not a diagnostic charting or
/// clinical record system. Every text field this package adds is
/// scheduling/billing-facing, never a clinical note.
///
/// "Provider Profile linked to User/Contact" relates to Contact, not
/// User, for the same reason Professional Services' Resource Assignment
/// doesn't relate to `User` either - see this module's own doc comment.
///
/// Patient's own suggested statuses (Active/Inactive/Archived) happen to
/// be *exactly* `CUSTOM_RECORD_STATUSES`, the one time in this module a
/// custom object's built-in status vocabulary needs no select-field
/// workaround at all - Patient Profile drives its lifecycle straight off
/// the built-in status/`status_changed` trigger, unlike every stage field
/// elsewhere in this file.
pub fn practice_admin_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.practice_admin",
        "name": "Dental & Clinic Practice Administration",
        "industry": "Healthcare",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "patient_profile", "singular_label": "Patient Profile", "plural_label": "Patient Profiles", "icon": "🪪", "prefix": "PAT", "digits": 4 },
            { "key": "provider_profile", "singular_label": "Provider Profile", "plural_label": "Provider Profiles", "icon": "🦷", "prefix": "PROV", "digits": 4 },
            { "key": "appointment", "singular_label": "Appointment", "plural_label": "Appointments", "icon": "📅", "prefix": "APPT", "digits": 5 },
            { "key": "treatment_plan", "singular_label": "Treatment Plan", "plural_label": "Treatment Plans", "icon": "📋", "prefix": "TXP", "digits": 4 },
            { "key": "procedure_item", "singular_label": "Procedure Item", "plural_label": "Procedure Items", "icon": "🔧", "prefix": "PROC", "digits": 5 },
            { "key": "recall", "singular_label": "Recall", "plural_label": "Recalls", "icon": "🔔", "prefix": "RCL", "digits": 4 },
            { "key": "insurance_profile", "singular_label": "Insurance Profile", "plural_label": "Insurance Profiles", "icon": "🛡", "prefix": "INS", "digits": 4 },
            { "key": "billing_claim", "singular_label": "Billing Claim", "plural_label": "Billing Claims", "icon": "🧾", "prefix": "CLM", "digits": 5 }
        ],
        "fields": practice_admin_fields(),
        "relationships": practice_admin_relationships(),
        "business_rules": practice_admin_business_rules(),
        "workflows": practice_admin_workflows(),
        "screen_layouts": [
            {
                "entity_type": "patient_profile",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["communication_preference", "non_clinical_notes"] }
                            ],
                            // Indices into `relationships` below: Appointments (2), Treatment Plans (4), Recalls (7), Insurance Profiles (8).
                            "related": ["2", "4", "7", "8"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Appointments by Stage", "entity_type": "appointment", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Practice Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Practice Administration",
            "icon": "🦷",
            "description": "Patients, providers, appointments, treatment plans and recalls for dental offices and small clinics - administrative scheduling and billing, not clinical charting.",
            "object_keys": [
                "patient_profile", "provider_profile", "appointment", "treatment_plan", "procedure_item", "recall", "insurance_profile", "billing_claim", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names (Practice Admin, Provider, Reception, Billing) onto this build's
            // actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "editor" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object exists here either - every object is a
        // live scheduling/billing record.
        "seed_data": []
    })
    .to_string()
}

fn practice_admin_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        patient_profile_fields(),
        provider_profile_fields(),
        appointment_fields(),
        treatment_plan_fields(),
        procedure_item_fields(),
        recall_fields(),
        insurance_profile_fields(),
        billing_claim_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn patient_profile_fields() -> serde_json::Value {
    json!([
        { "key": "communication_preference", "entity_type": "patient_profile", "label": "Communication Preference", "field_type": "select", "options": ["Email", "Phone", "SMS", "Mail"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Email", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "non_clinical_notes", "entity_type": "patient_profile", "label": "Non-Clinical Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Administrative flags only - e.g. wheelchair access, translator needed. Not a clinical record.", "placeholder": null }
    ])
}

fn provider_profile_fields() -> serde_json::Value {
    json!([
        { "key": "provider_type", "entity_type": "provider_profile", "label": "Provider Type", "field_type": "select", "options": ["Dentist", "Hygienist", "Dental Assistant", "Specialist"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Dentist", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "license_reference", "entity_type": "provider_profile", "label": "License / Reference ID", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": true, "help_text": null, "placeholder": null }
    ])
}

fn appointment_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "appointment", "label": "Stage", "field_type": "select", "options": ["Requested", "Confirmed", "Checked In", "In Progress", "Completed", "No Show", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Requested", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "appt_type", "entity_type": "appointment", "label": "Type", "field_type": "select", "options": ["Checkup", "Cleaning", "Filling", "Extraction", "Consultation", "Other"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Checkup", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "appt_date", "entity_type": "appointment", "label": "Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Free text, not a real time value - see this module's own doc comment on the
        // missing time-of-day field type.
        { "key": "start_time_text", "entity_type": "appointment", "label": "Start Time", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "e.g. 9:00 AM", "placeholder": null },
        { "key": "duration_minutes", "entity_type": "appointment", "label": "Duration (minutes)", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "30", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reason", "entity_type": "appointment", "label": "Reason", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed by the "Complete appointment" business rule below -
        // approximates "completion timestamp" with a date, for the same reason as start_time_text.
        { "key": "completed_date", "entity_type": "appointment", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn treatment_plan_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "treatment_plan", "label": "Stage", "field_type": "select", "options": ["Draft", "Presented", "Accepted", "Partially Accepted", "Completed", "Declined"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "plan_date", "entity_type": "treatment_plan", "label": "Plan Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "estimated_amount", "entity_type": "treatment_plan", "label": "Estimated Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn procedure_item_fields() -> serde_json::Value {
    json!([
        { "key": "service_code", "entity_type": "procedure_item", "label": "Service Code", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "tooth_site", "entity_type": "procedure_item", "label": "Tooth / Site", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "fee", "entity_type": "procedure_item", "label": "Fee", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "procedure_item", "label": "Stage", "field_type": "select", "options": ["Planned", "Accepted", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn recall_fields() -> serde_json::Value {
    json!([
        { "key": "recall_type", "entity_type": "recall", "label": "Recall Type", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "due_date", "entity_type": "recall", "label": "Due Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Reminder automation isn't available for a custom object's own date fields - see this module's own doc comment.", "placeholder": null },
        { "key": "stage", "entity_type": "recall", "label": "Stage", "field_type": "select", "options": ["Due", "Contacted", "Scheduled", "Completed", "Deferred"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Due", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn insurance_profile_fields() -> serde_json::Value {
    json!([
        { "key": "payer_name", "entity_type": "insurance_profile", "label": "Payer Name", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "member_reference", "entity_type": "insurance_profile", "label": "Member / Reference ID", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn billing_claim_fields() -> serde_json::Value {
    json!([
        { "key": "claim_status", "entity_type": "billing_claim", "label": "Status", "field_type": "select", "options": ["Draft", "Submitted", "Paid", "Denied"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "submitted_date", "entity_type": "billing_claim", "label": "Submitted Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "claimed_amount", "entity_type": "billing_claim", "label": "Claimed Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Both required-when-Paid by the "Claim payment" business rule below.
        { "key": "paid_amount", "entity_type": "billing_claim", "label": "Paid Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "payment_date", "entity_type": "billing_claim", "label": "Payment Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Denied by the "Claim denial notes" business rule below.
        { "key": "denial_reason", "entity_type": "billing_claim", "label": "Denial Reason", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` array
/// references these relationships by their position.
fn practice_admin_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "patient_profile", "target_entity_type": "Contact", "relationship_type": "one_to_one", "forward_label": "Contact", "reverse_label": "Patient Profile", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "provider_profile", "target_entity_type": "Contact", "relationship_type": "one_to_one", "forward_label": "Contact", "reverse_label": "Provider Profile", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 2 */ { "source_entity_type": "appointment", "target_entity_type": "patient_profile", "relationship_type": "many_to_one", "forward_label": "Patient", "reverse_label": "Appointments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 3 */ { "source_entity_type": "appointment", "target_entity_type": "provider_profile", "relationship_type": "many_to_one", "forward_label": "Provider", "reverse_label": "Appointments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 4 */ { "source_entity_type": "treatment_plan", "target_entity_type": "patient_profile", "relationship_type": "many_to_one", "forward_label": "Patient", "reverse_label": "Treatment Plans", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "treatment_plan", "target_entity_type": "provider_profile", "relationship_type": "many_to_one", "forward_label": "Provider", "reverse_label": "Treatment Plans", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "procedure_item", "target_entity_type": "treatment_plan", "relationship_type": "many_to_one", "forward_label": "Treatment Plan", "reverse_label": "Procedure Items", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 7 */ { "source_entity_type": "recall", "target_entity_type": "patient_profile", "relationship_type": "many_to_one", "forward_label": "Patient", "reverse_label": "Recalls", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 8 */ { "source_entity_type": "insurance_profile", "target_entity_type": "patient_profile", "relationship_type": "many_to_one", "forward_label": "Patient", "reverse_label": "Insurance Profiles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 9 */ { "source_entity_type": "appointment", "target_entity_type": "treatment_plan", "relationship_type": "many_to_one", "forward_label": "Treatment Plan", "reverse_label": "Appointments", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 10 */ { "source_entity_type": "billing_claim", "target_entity_type": "patient_profile", "relationship_type": "many_to_one", "forward_label": "Patient", "reverse_label": "Billing Claims", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 11 */ { "source_entity_type": "billing_claim", "target_entity_type": "appointment", "relationship_type": "many_to_one", "forward_label": "Appointment", "reverse_label": "Billing Claims", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 }
    ])
}

/// Of the spec's five business rules, only "Complete appointment"
/// survives as written. The rest all need something this module's own
/// doc comment already rules out: "Appointment validation"'s Patient/
/// Provider half and "Patient identity" both need a rule that can require
/// a relationship link exists, not just a field value (the field half of
/// "Appointment validation" - start time and duration - is instead
/// enforced the same way every other package enforces an always-required
/// field: `required: true` on the field definition itself, see
/// `appointment_fields`). "Schedule collision" needs interval-overlap
/// detection across every other appointment sharing a provider.
/// "Treatment acceptance" needs a count of a treatment plan's own active
/// Procedure Items - a cross-record aggregate.
fn practice_admin_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "appointment",
            "name": "Complete appointment",
            "description": "A completed appointment must record its completion date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "billing_claim",
            "name": "Claim payment",
            "description": "A paid claim must record its paid amount and payment date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Paid" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "paid_amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "payment_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "billing_claim",
            "name": "Claim denial notes",
            "description": "A denied claim must record its denial reason.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Denied" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "denial_reason", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Recall due" (spec: due-date-reached creates a contact task) needs a
/// `date_reached`-style trigger on a custom object's own date field,
/// already ruled out by this module's own doc comment. "Appointment
/// completed marks the related recall complete / creates the next recall"
/// is left out too - Appointment has no relationship to Recall at all in
/// this manifest (nothing in the spec's own relationship table connects
/// them directly), and the "if configured" half would need the
/// conditional-bulk-update capability this module's own doc comment
/// already covers is missing. "Treatment accepted" only creates a task,
/// not a Quote - `create_record` only ever creates a Company or an active
/// custom object (`is_creatable_entity_type`), never a Quote.
fn practice_admin_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "appointment",
            "name": "Appointment confirmation",
            "description": "A newly requested appointment gets a confirmation task for reception.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Confirm this appointment with the patient\",\"description\":null,\"due_in_days\":0,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "appointment",
            "name": "No-show follow-up",
            "description": "A missed appointment gets a follow-up task for reception.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "No Show" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up on missed appointment\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "treatment_plan",
            "name": "Treatment accepted",
            "description": "An accepted treatment plan gets a billing-preparation task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Accepted" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Prepare billing/quote for accepted treatment plan\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "billing_claim",
            "name": "Billing claim submitted",
            "description": "Submitting a billing claim opens a task to track it with the payer.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "claim_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Submitted" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Track submitted billing claim\",\"description\":null,\"due_in_days\":14,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.recruitment ---------------------------------------------------

/// `lanesra.recruitment` v1.0.0 - the sixth package, sequenced right
/// after Dental/Clinic Practice Administration per the dev spec. See
/// this module's own doc comment for what's included and deliberately
/// left out.
///
/// The spec's "Skill" object is named `competency` here, not `skill` -
/// Field Service already claims the `skill` custom object key
/// workspace-wide, the same "Project"/"engagement" collision this
/// module's own doc comment already covers for Construction &
/// Contractors and Professional Services.
///
/// Interviewer isn't its own field - like Professional Services'
/// Resource Assignment and Practice Administration's Provider Profile,
/// there's no relationship-capable `User` entity type, so Interview's
/// built-in owner field stands in for "who's interviewing" instead.
pub fn recruitment_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.recruitment",
        "name": "Recruitment & Staffing",
        "industry": "Staffing",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "job_requisition", "singular_label": "Job", "plural_label": "Jobs", "icon": "📌", "prefix": "JOB", "digits": 4 },
            { "key": "candidate_profile", "singular_label": "Candidate Profile", "plural_label": "Candidate Profiles", "icon": "🧑‍💼", "prefix": "CAND", "digits": 4 },
            { "key": "application", "singular_label": "Application", "plural_label": "Applications", "icon": "📨", "prefix": "APP", "digits": 5 },
            { "key": "interview", "singular_label": "Interview", "plural_label": "Interviews", "icon": "🎙", "prefix": "INT", "digits": 5 },
            { "key": "offer", "singular_label": "Offer", "plural_label": "Offers", "icon": "🤝", "prefix": "OFR", "digits": 4 },
            { "key": "placement", "singular_label": "Placement", "plural_label": "Placements", "icon": "✅", "prefix": "PLC", "digits": 4 },
            { "key": "competency", "singular_label": "Skill", "plural_label": "Skills", "icon": "🏅", "prefix": "SKL", "digits": 3 },
            { "key": "candidate_skill", "singular_label": "Candidate Skill", "plural_label": "Candidate Skills", "icon": "🔗", "prefix": "CSK", "digits": 5 },
            { "key": "reference_check", "singular_label": "Reference Check", "plural_label": "Reference Checks", "icon": "📞", "prefix": "REF", "digits": 5 }
        ],
        "fields": recruitment_fields(),
        "relationships": recruitment_relationships(),
        "business_rules": recruitment_business_rules(),
        "workflows": recruitment_workflows(),
        "screen_layouts": [
            {
                "entity_type": "application",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["stage", "submitted_date", "score", "disposition"] }
                            ],
                            // Indices into `relationships` below: Interviews (5), Offers (6), Placement (7).
                            "related": ["5", "6", "7"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Applications by Stage", "entity_type": "application", "group_by_source": "custom", "group_by_field": "stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Recruiting Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Recruitment & Staffing",
            "icon": "🧑‍💼",
            "description": "Jobs, candidates, applications, interviews, offers and placements for recruiting agencies and SMB talent teams.",
            "object_keys": [
                "job_requisition", "candidate_profile", "application", "interview", "offer", "placement", "competency", "candidate_skill", "reference_check", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names (Recruitment Admin, Recruiter, Recruiting Manager, Account
            // Manager, Coordinator) onto this build's actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object exists here either - even Skill is
        // populated as recruiters build out their own taxonomy, not shipped
        // pre-seeded.
        "seed_data": []
    })
    .to_string()
}

fn recruitment_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        job_requisition_fields(),
        candidate_profile_fields(),
        application_fields(),
        interview_fields(),
        offer_fields(),
        placement_fields(),
        competency_fields(),
        candidate_skill_fields(),
        reference_check_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn job_requisition_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "job_requisition", "label": "Stage", "field_type": "select", "options": ["Draft", "Open", "On Hold", "Filled", "Closed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "title", "entity_type": "job_requisition", "label": "Title", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "location", "entity_type": "job_requisition", "label": "Location", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "openings", "entity_type": "job_requisition", "label": "Openings", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": "1", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "rate_or_salary", "entity_type": "job_requisition", "label": "Rate / Salary", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn candidate_profile_fields() -> serde_json::Value {
    json!([
        { "key": "skills_summary", "entity_type": "candidate_profile", "label": "Skills Summary", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "location", "entity_type": "candidate_profile", "label": "Location", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "source", "entity_type": "candidate_profile", "label": "Source", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "availability", "entity_type": "candidate_profile", "label": "Availability", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "consent_flags", "entity_type": "candidate_profile", "label": "Consent to Contact", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn application_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "application", "label": "Stage", "field_type": "select", "options": ["Sourced", "Screening", "Submitted", "Interview", "Offer", "Placed", "Rejected", "Withdrawn"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Sourced", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "submitted_date", "entity_type": "application", "label": "Submitted Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "score", "entity_type": "application", "label": "Score", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "disposition", "entity_type": "application", "label": "Disposition", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn interview_fields() -> serde_json::Value {
    json!([
        { "key": "interview_type", "entity_type": "interview", "label": "Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // See this module's own doc comment on the missing time-of-day field type.
        { "key": "scheduled_date", "entity_type": "interview", "label": "Scheduled Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "outcome", "entity_type": "interview", "label": "Outcome", "field_type": "select", "options": ["Scheduled", "Completed", "Cancelled", "No Show"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Scheduled", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "feedback_status", "entity_type": "interview", "label": "Feedback Status", "field_type": "select", "options": ["Pending", "Submitted"], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn offer_fields() -> serde_json::Value {
    json!([
        { "key": "amount", "entity_type": "offer", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "start_date", "entity_type": "offer", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "stage", "entity_type": "offer", "label": "Stage", "field_type": "select", "options": ["Draft", "Sent", "Accepted", "Rejected", "Withdrawn"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn placement_fields() -> serde_json::Value {
    json!([
        { "key": "stage", "entity_type": "placement", "label": "Stage", "field_type": "select", "options": ["Planned", "Active", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Active by the "Placement start" business rule below - a
        // re-scoping of the spec's "Placement validation" rule onto the object that
        // actually holds a start date (see this file's own doc comment on why the
        // spec's own Application-side version of this rule isn't implementable).
        { "key": "start_date", "entity_type": "placement", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "placement", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "fee_or_rate", "entity_type": "placement", "label": "Fee / Rate", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn competency_fields() -> serde_json::Value {
    json!([
        { "key": "category", "entity_type": "competency", "label": "Category", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn candidate_skill_fields() -> serde_json::Value {
    json!([
        { "key": "proficiency", "entity_type": "candidate_skill", "label": "Proficiency", "field_type": "select", "options": ["Beginner", "Intermediate", "Advanced", "Expert"], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Beginner", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "years_experience", "entity_type": "candidate_skill", "label": "Years of Experience", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn reference_check_fields() -> serde_json::Value {
    json!([
        { "key": "check_status", "entity_type": "reference_check", "label": "Status", "field_type": "select", "options": ["Requested", "Completed", "Failed"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Requested", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reference_name", "entity_type": "reference_check", "label": "Reference Name", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed/Failed by the "Reference check completion" business rule below.
        { "key": "completed_date", "entity_type": "reference_check", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "notes", "entity_type": "reference_check", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` and the
/// Offer-Accepted workflow's `update_related_record`/`create_record`
/// actions reference these relationships by their position in this
/// array. The last one (Placement <-> Offer) exists purely to give that
/// workflow's `create_record` action a relationship to link the new
/// Placement back to the Offer that spawned it - the same "add one
/// relationship purely to support a create_record workflow" pattern
/// Construction & Contractors and Professional Services both used for
/// their own Opportunity link.
fn recruitment_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "job_requisition", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Jobs", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "job_requisition", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Hiring Contact", "reverse_label": "Jobs", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 2 */ { "source_entity_type": "candidate_profile", "target_entity_type": "Contact", "relationship_type": "one_to_one", "forward_label": "Contact", "reverse_label": "Candidate Profile", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 3 */ { "source_entity_type": "application", "target_entity_type": "candidate_profile", "relationship_type": "many_to_one", "forward_label": "Candidate", "reverse_label": "Applications", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "application", "target_entity_type": "job_requisition", "relationship_type": "many_to_one", "forward_label": "Job", "reverse_label": "Applications", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "interview", "target_entity_type": "application", "relationship_type": "many_to_one", "forward_label": "Application", "reverse_label": "Interviews", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 6 */ { "source_entity_type": "offer", "target_entity_type": "application", "relationship_type": "many_to_one", "forward_label": "Application", "reverse_label": "Offers", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 7 */ { "source_entity_type": "placement", "target_entity_type": "application", "relationship_type": "many_to_one", "forward_label": "Application", "reverse_label": "Placement", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 8 */ { "source_entity_type": "placement", "target_entity_type": "candidate_profile", "relationship_type": "many_to_one", "forward_label": "Candidate", "reverse_label": "Placements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 9 */ { "source_entity_type": "placement", "target_entity_type": "job_requisition", "relationship_type": "many_to_one", "forward_label": "Job", "reverse_label": "Placements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 10 */ { "source_entity_type": "placement", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Customer", "reverse_label": "Placements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 11 */ { "source_entity_type": "candidate_skill", "target_entity_type": "candidate_profile", "relationship_type": "many_to_one", "forward_label": "Candidate", "reverse_label": "Skills", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 12 */ { "source_entity_type": "candidate_skill", "target_entity_type": "competency", "relationship_type": "many_to_one", "forward_label": "Skill", "reverse_label": "Candidates", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        // Exists purely so the "Offer accepted" workflow's create_record action has a
        // relationship connecting its trigger entity (offer) to the entity it creates
        // (placement) - index 7 above (Placement <-> Application) is the spec's own
        // relationship and doesn't connect to Offer at all.
        /* 13 */ { "source_entity_type": "placement", "target_entity_type": "offer", "relationship_type": "many_to_one", "forward_label": "Offer", "reverse_label": "Placement Draft", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 14 */ { "source_entity_type": "reference_check", "target_entity_type": "candidate_profile", "relationship_type": "many_to_one", "forward_label": "Candidate", "reverse_label": "Reference Checks", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 15 */ { "source_entity_type": "reference_check", "target_entity_type": "application", "relationship_type": "many_to_one", "forward_label": "Application", "reverse_label": "Reference Checks", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 }
    ])
}

/// The spec's four business rules are all either cross-record checks
/// this module's own doc comment already rules out ("Candidate
/// duplicate" and "Application uniqueness" both need scanning every
/// other record of the same type for a match) or need a capability nothing
/// else in this file needed yet ("Interview scheduling"'s "must be in
/// the future" needs comparing against today's date, not a literal or
/// another field). "Placement validation" (spec: on Application ->
/// Placed, require an accepted Offer, a Start Date and a Customer) is
/// re-scoped below into "Placement start requires a start date" - the
/// Start Date the spec actually means lives on Placement, not
/// Application, and "an accepted Offer" is the same cross-record gap
/// as the other two.
fn recruitment_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "placement",
            "name": "Placement start",
            "description": "An active placement must record its start date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Active" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "start_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "reference_check",
            "name": "Reference check completion",
            "description": "A completed or failed reference check must record its completed date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "check_status", "operator": "in_list", "value": "Completed|Failed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Job filled" (spec: filled placements >= openings optionally sets Job
/// -> Filled) needs a count of a job's own linked Placements, the same
/// cross-record aggregate gap already covered above. "Placement start"
/// (spec: Placement Start Date reached creates a check-in task) needs a
/// `date_reached`-style trigger on a custom object's own date field,
/// already ruled out too.
fn recruitment_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "application",
            "name": "Application interview stage",
            "description": "Moving an application to Interview creates a scheduling task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Interview" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Schedule interview\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "interview",
            "name": "Interview scheduled",
            "description": "A newly created interview notifies the recruiting team.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "add_notification", "params_json": "{\"message\":\"An interview was scheduled\",\"audience\":\"all_admins\"}" }
            ]
        },
        {
            "entity_type": "offer",
            "name": "Offer accepted",
            "description": "Accepting an offer moves its application to Placed and opens a Placement draft linked back to the offer.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "stage", "operator": "equals", "value": "Accepted" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":6,\"target_field_key\":\"stage\",\"target_field_source\":\"custom\",\"value\":\"Placed\",\"copy_from_field_key\":null}" },
                { "action_type": "create_record", "params_json": "{\"entity_type\":\"placement\",\"relationship_ref\":13,\"name_template\":null}" }
            ]
        },
        {
            "entity_type": "reference_check",
            "name": "Reference check requested",
            "description": "Logging a new reference check opens a task to contact the reference.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Contact reference\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

/// The seventh package: a lightweight brokerage data model for real-estate
/// agents and small brokerages managing properties, listings, buyers/
/// sellers, viewings, offers and transactions.
///
/// Key-collision note (see this module's own doc comment): the spec's
/// "Property" object would collide with Property Management's own
/// `property` custom object key, and its "Offer" would collide with
/// Recruitment's `offer` - renamed to `listing_property` and
/// `purchase_offer` respectively so all packages coexist in one
/// workspace.
///
/// "Agent Profile | Custom extension linked to User" and Listing's
/// "listing agent" both use the same "no relationship-capable `User`
/// entity type" workaround this file has used repeatedly (Professional
/// Services' Resource Assignment, Practice Administration's Provider
/// Profile, Recruitment's Interview): the linked user is the record's
/// own built-in `owner_user_id`, not a relationship_definition - so
/// `agent_profile` carries no relationship of its own at all.
pub fn real_estate_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.real_estate",
        "name": "Real Estate Brokerage",
        "industry": "Real Estate",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "listing_property", "singular_label": "Property", "plural_label": "Properties", "icon": "🏠", "prefix": "PROP", "digits": 4 },
            { "key": "listing", "singular_label": "Listing", "plural_label": "Listings", "icon": "📋", "prefix": "LST", "digits": 4 },
            { "key": "showing", "singular_label": "Showing", "plural_label": "Showings", "icon": "👁", "prefix": "SHW", "digits": 5 },
            { "key": "purchase_offer", "singular_label": "Offer", "plural_label": "Offers", "icon": "🤝", "prefix": "OFFR", "digits": 4 },
            { "key": "transaction", "singular_label": "Transaction", "plural_label": "Transactions", "icon": "🏆", "prefix": "TXN", "digits": 4 },
            { "key": "client_role", "singular_label": "Client Role", "plural_label": "Client Roles", "icon": "🔗", "prefix": "ROLE", "digits": 5 },
            { "key": "agent_profile", "singular_label": "Agent Profile", "plural_label": "Agent Profiles", "icon": "🧑‍💼", "prefix": "AGT", "digits": 3 },
            { "key": "commission_disbursement", "singular_label": "Commission Disbursement", "plural_label": "Commission Disbursements", "icon": "💰", "prefix": "DSB", "digits": 5 }
        ],
        "fields": real_estate_fields(),
        "relationships": real_estate_relationships(),
        "business_rules": real_estate_business_rules(),
        "workflows": real_estate_workflows(),
        "screen_layouts": [
            {
                "entity_type": "listing",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["listing_stage", "list_price", "list_date", "end_date"] }
                            ],
                            // Indices into `relationships` below: Showings (2), Offers (4), Client Roles (9).
                            "related": ["2", "4", "9"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Listings by Stage", "entity_type": "listing", "group_by_source": "custom", "group_by_field": "listing_stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Brokerage Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Real Estate Brokerage",
            "icon": "🏘",
            "description": "Properties, listings, showings, offers and transactions for real-estate agents and small brokerages.",
            "object_keys": [
                "listing_property", "listing", "showing", "purchase_offer", "transaction", "client_role", "agent_profile", "commission_disbursement", "Contact", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // Spec role names (Broker/Admin, Agent, Transaction Coordinator,
            // Marketing Assistant, Read-only) mapped onto this build's actual
            // role set - see field_service_manifest_json's own note on this
            // same mapping.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        "seed_data": []
    })
    .to_string()
}

fn real_estate_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        listing_property_fields(),
        listing_fields(),
        showing_fields(),
        purchase_offer_fields(),
        transaction_fields(),
        client_role_fields(),
        agent_profile_fields(),
        commission_disbursement_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn listing_property_fields() -> serde_json::Value {
    json!([
        { "key": "property_type", "entity_type": "listing_property", "label": "Property Type", "field_type": "select", "options": ["House", "Condo", "Townhouse", "Land", "Multi-Family", "Commercial"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "House", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "bedrooms", "entity_type": "listing_property", "label": "Bedrooms", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "bathrooms", "entity_type": "listing_property", "label": "Bathrooms", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "size_sqft", "entity_type": "listing_property", "label": "Size (sq ft)", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "property_stage", "entity_type": "listing_property", "label": "Stage", "field_type": "select", "options": ["Prospect", "Available", "Under Contract", "Sold/Leased", "Off Market"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Prospect", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn listing_fields() -> serde_json::Value {
    json!([
        { "key": "listing_stage", "entity_type": "listing", "label": "Stage", "field_type": "select", "options": ["Draft", "Coming Soon", "Active", "Conditional", "Pending", "Closed", "Expired", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "list_price", "entity_type": "listing", "label": "List Price", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "list_date", "entity_type": "listing", "label": "List Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // See this module's own doc comment on the missing date_reached
        // trigger for custom objects - Listing expiry is checked by an
        // admin reading this field, not automated.
        { "key": "end_date", "entity_type": "listing", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn showing_fields() -> serde_json::Value {
    json!([
        { "key": "scheduled_date", "entity_type": "showing", "label": "Scheduled Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "outcome", "entity_type": "showing", "label": "Outcome", "field_type": "select", "options": ["Scheduled", "Completed", "Cancelled", "No Show"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Scheduled", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn purchase_offer_fields() -> serde_json::Value {
    json!([
        { "key": "offer_stage", "entity_type": "purchase_offer", "label": "Stage", "field_type": "select", "options": ["Draft", "Submitted", "Countered", "Accepted", "Rejected", "Expired", "Withdrawn"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "purchase_offer", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expiry_date", "entity_type": "purchase_offer", "label": "Expiry Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "conditions_text", "entity_type": "purchase_offer", "label": "Conditions", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn transaction_fields() -> serde_json::Value {
    json!([
        { "key": "transaction_status", "entity_type": "transaction", "label": "Status", "field_type": "select", "options": ["Pending", "Conditional", "Firm", "Closing", "Closed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "closing_date", "entity_type": "transaction", "label": "Closing Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "final_price", "entity_type": "transaction", "label": "Final Price", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "commission_summary", "entity_type": "transaction", "label": "Commission Summary", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn client_role_fields() -> serde_json::Value {
    json!([
        { "key": "role_type", "entity_type": "client_role", "label": "Role", "field_type": "select", "options": ["Buyer", "Seller", "Landlord", "Tenant"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Buyer", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn agent_profile_fields() -> serde_json::Value {
    json!([
        { "key": "license_number", "entity_type": "agent_profile", "label": "License Number", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": true, "help_text": null, "placeholder": null },
        { "key": "brokerage_name", "entity_type": "agent_profile", "label": "Brokerage", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "license_active", "entity_type": "agent_profile", "label": "License Active", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn commission_disbursement_fields() -> serde_json::Value {
    json!([
        { "key": "disbursement_status", "entity_type": "commission_disbursement", "label": "Status", "field_type": "select", "options": ["Pending", "Paid"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "commission_disbursement", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Paid by the "Disbursement payment" business rule below.
        { "key": "paid_date", "entity_type": "commission_disbursement", "label": "Paid Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related`, and the
/// "Offer accepted"/"Transaction closed" workflows' `update_related_record`/
/// `create_record` actions, reference these relationships by their
/// position in this array. Index 7 (Transaction -> Listing) exists purely
/// to give the "Transaction closed" workflow a direct relationship to
/// write to - the spec's own model only links Transaction to its
/// Offer, not to the Listing - the same "add one relationship purely to
/// support a create_record/update_related_record workflow" pattern
/// Construction & Contractors, Professional Services and Recruitment all
/// used for their own extra links.
///
/// `client_role` is deliberately scoped to Listing rather than the
/// spec's broader "Contact/Customer <-> transaction context" - a
/// relationship_definition names one specific target type, so "buyer
/// role on this listing" is what's actually buildable; a role's
/// relevance to the eventual Transaction is reachable transitively via
/// Listing -> Offer -> Transaction instead of a second direct link.
fn real_estate_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "listing", "target_entity_type": "listing_property", "relationship_type": "many_to_one", "forward_label": "Property", "reverse_label": "Listings", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "listing_property", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Owner/Seller", "reverse_label": "Properties", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "showing", "target_entity_type": "listing", "relationship_type": "many_to_one", "forward_label": "Listing", "reverse_label": "Showings", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "showing", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Buyer", "reverse_label": "Showings", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 4 */ { "source_entity_type": "purchase_offer", "target_entity_type": "listing", "relationship_type": "many_to_one", "forward_label": "Listing", "reverse_label": "Offers", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 5 */ { "source_entity_type": "purchase_offer", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Buyer", "reverse_label": "Offers", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "transaction", "target_entity_type": "purchase_offer", "relationship_type": "many_to_one", "forward_label": "Accepted Offer", "reverse_label": "Transaction", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 7 */ { "source_entity_type": "transaction", "target_entity_type": "listing", "relationship_type": "many_to_one", "forward_label": "Listing", "reverse_label": "Transactions", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 8 */ { "source_entity_type": "client_role", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Contact", "reverse_label": "Client Roles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 9 */ { "source_entity_type": "client_role", "target_entity_type": "listing", "relationship_type": "many_to_one", "forward_label": "Listing", "reverse_label": "Client Roles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 10 */ { "source_entity_type": "commission_disbursement", "target_entity_type": "transaction", "relationship_type": "many_to_one", "forward_label": "Transaction", "reverse_label": "Commission Disbursements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 11 */ { "source_entity_type": "commission_disbursement", "target_entity_type": "agent_profile", "relationship_type": "many_to_one", "forward_label": "Agent", "reverse_label": "Commission Disbursements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 }
    ])
}

/// The spec's four business rules include two this module's own doc
/// comment already rules out:
/// - "Offer acceptance" ("only one accepted active offer per Listing")
///   needs scanning every sibling Offer on the same Listing - the
///   overlap/conflict-detection gap.
/// - "Listing activation"'s "Property" and "agent" requirements are
///   relationship-existence and owner-field checks respectively, neither
///   of which a `require` action can target (see this module's own doc
///   comment) - only its plain-field requirements (price, start date)
///   are enforced below.
///
/// "Offer integrity" (spec: "Offer created -> require Listing, buyer
/// party, amount and expiry") is re-scoped the same way Recruitment's
/// "Placement validation" was: a business rule can't gate on "just
/// created" (only a saved-state condition), so this fires once the offer
/// leaves Draft instead - by then it should carry real numbers regardless
/// of whether this is its first save or a later edit. Listing/buyer are
/// relationship-existence checks, left unenforced by a rule for the same
/// reason as "Listing activation" above.
fn real_estate_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "purchase_offer",
            "name": "Offer integrity",
            "description": "An offer that has left Draft must record its amount and expiry.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "offer_stage", "operator": "not_equals", "value": "Draft" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "expiry_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "listing",
            "name": "Listing activation",
            "description": "An active listing must record its price and list date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "listing_stage", "operator": "equals", "value": "Active" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "list_price", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "list_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "transaction",
            "name": "Transaction close",
            "description": "A closed transaction must record its closing date and final price.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "transaction_status", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "closing_date", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "final_price", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "commission_disbursement",
            "name": "Disbursement payment",
            "description": "A paid commission disbursement must record its paid date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "disbursement_status", "operator": "equals", "value": "Paid" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "paid_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Listing expiry" (spec: end date reached AND still Active sets
/// Expired and creates a renewal task) needs a `date_reached`-style
/// trigger on a custom object's own date field, already ruled out by
/// this module's own doc comment - left out entirely rather than
/// approximated.
fn real_estate_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "showing",
            "name": "Showing scheduled",
            "description": "A newly created showing gets an agent follow-up task.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up after showing\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "purchase_offer",
            "name": "Offer accepted",
            "description": "Accepting an offer moves its listing to Pending and opens a Transaction linked back to the offer.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "offer_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "offer_stage", "operator": "equals", "value": "Accepted" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":4,\"target_field_key\":\"listing_stage\",\"target_field_source\":\"custom\",\"value\":\"Pending\",\"copy_from_field_key\":null}" },
                { "action_type": "create_record", "params_json": "{\"entity_type\":\"transaction\",\"relationship_ref\":6,\"name_template\":null}" }
            ]
        },
        {
            "entity_type": "transaction",
            "name": "Transaction closed",
            "description": "Closing a transaction closes its listing and opens a post-closing task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "transaction_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "transaction_status", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":7,\"target_field_key\":\"listing_stage\",\"target_field_source\":\"custom\",\"value\":\"Closed\",\"copy_from_field_key\":null}" },
                { "action_type": "create_task", "params_json": "{\"title\":\"Post-closing checklist\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "commission_disbursement",
            "name": "Commission disbursement created",
            "description": "Logging a new commission disbursement opens a task to process payment.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Process commission disbursement\",\"description\":null,\"due_in_days\":5,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

/// The eighth package: a matter-management and administrative billing
/// model for small law firms - client/matter operations, deadlines, time
/// and documents, deliberately not jurisdiction-specific court filing or
/// full trust accounting (spec's own stated v1 scope).
///
/// Key-collision note (see this module's own doc comment): the spec's
/// "Time Entry" and "Expense" would collide with Professional Services'
/// own `time_entry`/`expense` custom object keys - renamed to
/// `matter_time_entry`/`matter_expense`.
pub fn legal_practice_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.legal_practice",
        "name": "Legal Practice",
        "industry": "Legal Services",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "matter", "singular_label": "Matter", "plural_label": "Matters", "icon": "⚖", "prefix": "MAT", "digits": 4 },
            { "key": "matter_party", "singular_label": "Matter Party", "plural_label": "Matter Parties", "icon": "🔗", "prefix": "PTY", "digits": 5 },
            { "key": "matter_deadline", "singular_label": "Deadline", "plural_label": "Deadlines", "icon": "⏰", "prefix": "DL", "digits": 5 },
            { "key": "matter_time_entry", "singular_label": "Time Entry", "plural_label": "Time Entries", "icon": "⏱", "prefix": "TE", "digits": 5 },
            { "key": "matter_expense", "singular_label": "Expense", "plural_label": "Expenses", "icon": "💳", "prefix": "EXP", "digits": 5 },
            { "key": "trust_summary", "singular_label": "Trust Summary", "plural_label": "Trust Summaries", "icon": "🏦", "prefix": "TRS", "digits": 4 },
            { "key": "conflict_check", "singular_label": "Conflict Check", "plural_label": "Conflict Checks", "icon": "🔍", "prefix": "CFC", "digits": 5 }
        ],
        "fields": legal_practice_fields(),
        "relationships": legal_practice_relationships(),
        "business_rules": legal_practice_business_rules(),
        "workflows": legal_practice_workflows(),
        "screen_layouts": [
            {
                "entity_type": "matter",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["matter_stage", "matter_type", "opened_date", "closed_date"] }
                            ],
                            // Indices into `relationships` below: Matter Parties (1),
                            // Deadlines (3), Time Entries (4), Expenses (5), Invoices (6).
                            "related": ["1", "3", "4", "5", "6"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Matters by Stage", "entity_type": "matter", "group_by_source": "custom", "group_by_field": "matter_stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Legal Practice Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Legal Practice",
            "icon": "⚖️",
            "description": "Matters, parties, deadlines, time and expenses for small law firms.",
            "object_keys": [
                "matter", "matter_party", "matter_deadline", "matter_time_entry", "matter_expense", "trust_summary", "conflict_check", "Contact", "Invoice", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // Spec role names (Partner/Lawyer, Associate, Paralegal, Practice
            // Administrator, Billing) mapped onto this build's actual role
            // set - see field_service_manifest_json's own note on this same
            // mapping.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        "seed_data": []
    })
    .to_string()
}

fn legal_practice_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        matter_fields(),
        matter_party_fields(),
        matter_deadline_fields(),
        matter_time_entry_fields(),
        matter_expense_fields(),
        trust_summary_fields(),
        conflict_check_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn matter_fields() -> serde_json::Value {
    json!([
        { "key": "matter_stage", "entity_type": "matter", "label": "Stage", "field_type": "select", "options": ["Prospective", "Open", "On Hold", "Closing", "Closed", "Archived"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Prospective", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "matter_type", "entity_type": "matter", "label": "Matter Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "opened_date", "entity_type": "matter", "label": "Opened Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Closed by the "Matter close" business rule below.
        { "key": "closed_date", "entity_type": "matter", "label": "Closed Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "conflict_reference", "entity_type": "matter", "label": "Conflict Reference", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn matter_party_fields() -> serde_json::Value {
    json!([
        { "key": "role_type", "entity_type": "matter_party", "label": "Role", "field_type": "select", "options": ["Client", "Opposing Party", "Witness", "Counsel", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Client", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn matter_deadline_fields() -> serde_json::Value {
    json!([
        { "key": "deadline_type", "entity_type": "matter_deadline", "label": "Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // See this module's own doc comment on the missing date_reached
        // trigger for custom objects - the 7/2/1-day reminder workflow is
        // left out, but the due date itself is still tracked here.
        { "key": "due_date", "entity_type": "matter_deadline", "label": "Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "deadline_status", "entity_type": "matter_deadline", "label": "Status", "field_type": "select", "options": ["Open", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Open", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed by the "Deadline complete" business rule below.
        { "key": "completed_date", "entity_type": "matter_deadline", "label": "Completed Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn matter_time_entry_fields() -> serde_json::Value {
    json!([
        { "key": "entry_date", "entity_type": "matter_time_entry", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "hours", "entity_type": "matter_time_entry", "label": "Hours", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Submitted by the "Time entry description" business
        // rule below.
        { "key": "description", "entity_type": "matter_time_entry", "label": "Description", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "matter_time_entry", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "rate", "entity_type": "matter_time_entry", "label": "Rate", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "time_status", "entity_type": "matter_time_entry", "label": "Status", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Invoiced", "Rejected"], "required": true, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        // Written by the "Time approved" workflow below.
        { "key": "billing_status", "entity_type": "matter_time_entry", "label": "Billing Status", "field_type": "select", "options": ["Not Billed", "Eligible", "Billed"], "required": false, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Not Billed", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn matter_expense_fields() -> serde_json::Value {
    json!([
        { "key": "expense_date", "entity_type": "matter_expense", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "matter_expense", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "category", "entity_type": "matter_expense", "label": "Category", "field_type": "select", "options": ["Filing Fees", "Travel", "Expert Witness", "Copying", "Postage", "Other"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "matter_expense", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expense_status", "entity_type": "matter_expense", "label": "Status", "field_type": "select", "options": ["Draft", "Submitted", "Approved", "Invoiced", "Rejected"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn trust_summary_fields() -> serde_json::Value {
    json!([
        { "key": "balance", "entity_type": "trust_summary", "label": "Balance", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reference", "entity_type": "trust_summary", "label": "Reference", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn conflict_check_fields() -> serde_json::Value {
    json!([
        { "key": "check_status", "entity_type": "conflict_check", "label": "Status", "field_type": "select", "options": ["Pending", "Cleared", "Conflict Found"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "requested_date", "entity_type": "conflict_check", "label": "Requested Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Cleared by the "Conflict check resolution" business rule below.
        { "key": "cleared_date", "entity_type": "conflict_check", "label": "Cleared Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Conflict-Found by the "Conflict check notes" business rule below.
        { "key": "resolution_notes", "entity_type": "conflict_check", "label": "Resolution Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` list
/// references these relationships by their position in this array.
fn legal_practice_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "matter", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Client", "reverse_label": "Matters", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "matter_party", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Parties", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 2 */ { "source_entity_type": "matter_party", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Contact", "reverse_label": "Matter Roles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "matter_deadline", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Deadlines", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 4 */ { "source_entity_type": "matter_time_entry", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Time Entries", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 5 */ { "source_entity_type": "matter_expense", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Expenses", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 6 */ { "source_entity_type": "Invoice", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Invoices", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 7 */ { "source_entity_type": "conflict_check", "target_entity_type": "matter", "relationship_type": "many_to_one", "forward_label": "Matter", "reverse_label": "Conflict Checks", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 5 }
    ])
}

/// The spec's four business rules include two skipped or split for the
/// reasons this module's own doc comment already documents:
/// - "Matter close"'s "no open mandatory deadlines unless override" half
///   needs scanning every sibling Deadline on the same Matter - the
///   overlap/conflict-detection gap; only the plain-field "require Closed
///   Date" half is enforced below.
/// - "Time entry" (spec: hours > 0, Matter Open/Closing, description
///   required, all in one rule) is split into two rules below - "Matter
///   Open/Closing" is a cross-record read of a *different* record's field,
///   left out; the other two checks are independent conditions that can't
///   share one rule's `conditions` list without accidentally gating each
///   other (an `all` match would only require the description when hours
///   was *also* invalid), so they're two separate rules instead, the same
///   way Field Service split "Completion validation" into per-object
///   rules.
/// - "Matter party"'s "require role" is already guaranteed by the field
///   definition itself (`role_type` is `required: true` with a default),
///   so no separate rule is needed for it; its "prevent exact duplicate
///   active party relationship" half is the sibling-scanning gap, left
///   unenforced.
fn legal_practice_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "matter",
            "name": "Matter close",
            "description": "A closed matter must record its closed date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "matter_stage", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "closed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "matter_time_entry",
            "name": "Time entry hours",
            "description": "A submitted time entry must record more than zero hours.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "time_status", "operator": "equals", "value": "Submitted" },
                { "field_source": "custom", "field_key": "hours", "operator": "less_than", "value": "0.01" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Enter more than zero hours before submitting a time entry." }
            ]
        },
        {
            "entity_type": "matter_time_entry",
            "name": "Time entry description",
            "description": "A submitted time entry must record a description.",
            "match_type": "all",
            "priority": 1,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "time_status", "operator": "equals", "value": "Submitted" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "description", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "matter_deadline",
            "name": "Deadline complete",
            "description": "A completed deadline must record its completion date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "deadline_status", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completed_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "conflict_check",
            "name": "Conflict check resolution",
            "description": "A cleared conflict check must record its cleared date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "check_status", "operator": "equals", "value": "Cleared" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "cleared_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "conflict_check",
            "name": "Conflict check notes",
            "description": "A conflict check that found a conflict must record resolution notes.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "check_status", "operator": "equals", "value": "Conflict Found" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "resolution_notes", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// "Deadline approaching" (spec: due in 7/2/1 days creates/notifies owner
/// tasks) needs a `date_reached`-style trigger on a custom object's own
/// date field, already ruled out by this module's own doc comment - left
/// out entirely rather than approximated. "Matter closed"'s "archive open
/// non-required reminders" half needs a conditional bulk update through a
/// relationship (only the non-required ones), the same gap Construction &
/// Contractors' "Project close" workflow hit - only its "create final
/// billing review task" half is automated below.
fn legal_practice_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "matter",
            "name": "New matter",
            "description": "Opening a matter creates a standard matter-opening checklist task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "matter_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "matter_stage", "operator": "equals", "value": "Open" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Matter opening checklist\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "matter_time_entry",
            "name": "Time approved",
            "description": "Approved time is marked eligible for billing.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "time_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "time_status", "operator": "equals", "value": "Approved" }
            ],
            "actions": [
                { "action_type": "update_field", "params_json": "{\"target_field_key\":\"billing_status\",\"target_field_source\":\"custom\",\"value\":\"Eligible\",\"copy_from_field_key\":null}" }
            ]
        },
        {
            "entity_type": "matter",
            "name": "Matter closing",
            "description": "Moving a matter to Closing creates closing checklist tasks.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "matter_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 1,
            "conditions": [
                { "field_source": "custom", "field_key": "matter_stage", "operator": "equals", "value": "Closing" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Closing checklist\",\"description\":null,\"due_in_days\":7,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "matter",
            "name": "Matter closed",
            "description": "Closing a matter opens a final billing review task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "matter_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 2,
            "conditions": [
                { "field_source": "custom", "field_key": "matter_stage", "operator": "equals", "value": "Closed" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Final billing review\",\"description\":null,\"due_in_days\":5,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "conflict_check",
            "name": "Conflict check requested",
            "description": "Logging a new conflict check opens a task to run it.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Run conflict check\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

/// The ninth package: a constituent, membership, donation and
/// program-management model for small nonprofits, clubs, chambers,
/// associations and community organizations.
pub fn nonprofit_association_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.nonprofit_association",
        "name": "Nonprofit & Association",
        "industry": "Nonprofit",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "constituent_profile", "singular_label": "Constituent Profile", "plural_label": "Constituent Profiles", "icon": "👤", "prefix": "CONST", "digits": 4 },
            { "key": "membership_plan", "singular_label": "Membership Plan", "plural_label": "Membership Plans", "icon": "📇", "prefix": "PLAN", "digits": 3 },
            { "key": "membership", "singular_label": "Membership", "plural_label": "Memberships", "icon": "🪪", "prefix": "MBR", "digits": 5 },
            { "key": "donation", "singular_label": "Donation", "plural_label": "Donations", "icon": "💝", "prefix": "DON", "digits": 5 },
            { "key": "campaign", "singular_label": "Campaign", "plural_label": "Campaigns", "icon": "📣", "prefix": "CAMP", "digits": 4 },
            { "key": "program", "singular_label": "Program", "plural_label": "Programs", "icon": "📚", "prefix": "PRG", "digits": 4 },
            { "key": "program_participation", "singular_label": "Program Participation", "plural_label": "Program Participations", "icon": "🔗", "prefix": "PPT", "digits": 5 },
            { "key": "volunteer_assignment", "singular_label": "Volunteer Assignment", "plural_label": "Volunteer Assignments", "icon": "🙋", "prefix": "VOL", "digits": 5 },
            { "key": "event", "singular_label": "Event", "plural_label": "Events", "icon": "🎫", "prefix": "EVT", "digits": 4 },
            { "key": "grant", "singular_label": "Grant", "plural_label": "Grants", "icon": "🏛", "prefix": "GRT", "digits": 4 }
        ],
        "fields": nonprofit_association_fields(),
        "relationships": nonprofit_association_relationships(),
        "business_rules": nonprofit_association_business_rules(),
        "workflows": nonprofit_association_workflows(),
        "screen_layouts": [
            {
                "entity_type": "constituent_profile",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["constituent_type", "engagement_status", "preferences"] }
                            ],
                            // Indices into `relationships` below: Memberships (1),
                            // Donations (3), Program Participations (5).
                            "related": ["1", "3", "5"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Memberships by Stage", "entity_type": "membership", "group_by_source": "custom", "group_by_field": "membership_stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Nonprofit Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Nonprofit & Association",
            "icon": "🤝",
            "description": "Constituents, memberships, donations, campaigns, programs, volunteers and events for small nonprofits and associations.",
            "object_keys": [
                "constituent_profile", "membership_plan", "membership", "donation", "campaign", "program", "program_participation", "volunteer_assignment", "event", "grant", "Contact", "Invoice", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // Spec role names (Executive/Admin, Membership Manager,
            // Fundraising, Program Coordinator, Volunteer Coordinator)
            // mapped onto this build's actual role set - see
            // field_service_manifest_json's own note on this same mapping.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        "seed_data": []
    })
    .to_string()
}

fn nonprofit_association_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        constituent_profile_fields(),
        membership_plan_fields(),
        membership_fields(),
        donation_fields(),
        campaign_fields(),
        program_fields(),
        program_participation_fields(),
        volunteer_assignment_fields(),
        event_fields(),
        grant_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn constituent_profile_fields() -> serde_json::Value {
    json!([
        { "key": "constituent_type", "entity_type": "constituent_profile", "label": "Constituent Type", "field_type": "select", "options": ["Individual", "Organization", "Family", "Business"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Individual", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "engagement_status", "entity_type": "constituent_profile", "label": "Engagement Status", "field_type": "select", "options": ["Prospect", "Active", "Lapsed", "Inactive"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Prospect", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "preferences", "entity_type": "constituent_profile", "label": "Preferences", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn membership_plan_fields() -> serde_json::Value {
    json!([
        { "key": "duration_months", "entity_type": "membership_plan", "label": "Duration (months)", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "1", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "12", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "fee", "entity_type": "membership_plan", "label": "Fee", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "benefits_summary", "entity_type": "membership_plan", "label": "Benefits Summary", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "plan_active", "entity_type": "membership_plan", "label": "Active", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn membership_fields() -> serde_json::Value {
    json!([
        { "key": "membership_stage", "entity_type": "membership", "label": "Stage", "field_type": "select", "options": ["Pending", "Active", "Grace Period", "Expired", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Active by the "Membership activation" business rule below.
        { "key": "start_date", "entity_type": "membership", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "end_date", "entity_type": "membership", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Blocked-if-earlier-than start_date by the "Renewal integrity"
        // business rule below.
        { "key": "renewal_date", "entity_type": "membership", "label": "Renewal Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn donation_fields() -> serde_json::Value {
    json!([
        { "key": "donation_date", "entity_type": "donation", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Must be more than zero by the "Donation validation" business rule below.
        { "key": "amount", "entity_type": "donation", "label": "Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "donation_type", "entity_type": "donation", "label": "Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "donation_status", "entity_type": "donation", "label": "Status", "field_type": "select", "options": ["Pending", "Completed", "Refunded"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn campaign_fields() -> serde_json::Value {
    json!([
        { "key": "goal_amount", "entity_type": "campaign", "label": "Goal Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "campaign_start", "entity_type": "campaign", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "campaign_end", "entity_type": "campaign", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "campaign_status", "entity_type": "campaign", "label": "Status", "field_type": "select", "options": ["Planned", "Active", "Closed"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn program_fields() -> serde_json::Value {
    json!([
        { "key": "program_start", "entity_type": "program", "label": "Start Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "program_end", "entity_type": "program", "label": "End Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "program_status", "entity_type": "program", "label": "Status", "field_type": "select", "options": ["Planned", "Active", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Planned", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn program_participation_fields() -> serde_json::Value {
    json!([
        { "key": "participation_role", "entity_type": "program_participation", "label": "Role", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "participation_status", "entity_type": "program_participation", "label": "Status", "field_type": "select", "options": ["Registered", "Active", "Completed", "Withdrawn"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Registered", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn volunteer_assignment_fields() -> serde_json::Value {
    json!([
        { "key": "assignment_role", "entity_type": "volunteer_assignment", "label": "Role", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "hours", "entity_type": "volunteer_assignment", "label": "Hours", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "assignment_date", "entity_type": "volunteer_assignment", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn event_fields() -> serde_json::Value {
    json!([
        { "key": "event_date", "entity_type": "event", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "location", "entity_type": "event", "label": "Location", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Compared against registration counts by the spec's "Capacity
        // rule" - left unenforced, see this module's own doc comment on
        // cross-record aggregates.
        { "key": "capacity", "entity_type": "event", "label": "Capacity", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "event_status", "entity_type": "event", "label": "Status", "field_type": "select", "options": ["Draft", "Open", "Full", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn grant_fields() -> serde_json::Value {
    json!([
        { "key": "grant_status", "entity_type": "grant", "label": "Status", "field_type": "select", "options": ["Applied", "Awarded", "Declined", "Closed"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Applied", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "applied_date", "entity_type": "grant", "label": "Applied Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Both required-when-Awarded by the "Grant award" business rule below.
        { "key": "awarded_amount", "entity_type": "grant", "label": "Awarded Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "award_date", "entity_type": "grant", "label": "Award Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` list
/// references these relationships by their position in this array.
/// `constituent_profile`'s own link to `Contact` is `is_required: false`
/// (spec: "1:1 optional") - a Contact only becomes a constituent once an
/// admin deliberately links the two, matching the spec's "extension/role
/// to prevent duplicate master records" framing.
fn nonprofit_association_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "constituent_profile", "target_entity_type": "Contact", "relationship_type": "one_to_one", "forward_label": "Contact", "reverse_label": "Constituent Profile", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "membership", "target_entity_type": "constituent_profile", "relationship_type": "many_to_one", "forward_label": "Constituent", "reverse_label": "Memberships", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "membership", "target_entity_type": "membership_plan", "relationship_type": "many_to_one", "forward_label": "Plan", "reverse_label": "Memberships", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 3 */ { "source_entity_type": "donation", "target_entity_type": "constituent_profile", "relationship_type": "many_to_one", "forward_label": "Constituent", "reverse_label": "Donations", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 4 */ { "source_entity_type": "donation", "target_entity_type": "campaign", "relationship_type": "many_to_one", "forward_label": "Campaign", "reverse_label": "Donations", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 5 */ { "source_entity_type": "program_participation", "target_entity_type": "constituent_profile", "relationship_type": "many_to_one", "forward_label": "Constituent", "reverse_label": "Program Participations", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "program_participation", "target_entity_type": "program", "relationship_type": "many_to_one", "forward_label": "Program", "reverse_label": "Participants", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 7 */ { "source_entity_type": "volunteer_assignment", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Volunteer", "reverse_label": "Volunteer Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 8 */ { "source_entity_type": "volunteer_assignment", "target_entity_type": "program", "relationship_type": "many_to_one", "forward_label": "Program", "reverse_label": "Volunteer Assignments", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 9 */ { "source_entity_type": "volunteer_assignment", "target_entity_type": "event", "relationship_type": "many_to_one", "forward_label": "Event", "reverse_label": "Volunteer Assignments", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 10 */ { "source_entity_type": "Invoice", "target_entity_type": "membership", "relationship_type": "many_to_one", "forward_label": "Membership", "reverse_label": "Invoices", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 11 */ { "source_entity_type": "grant", "target_entity_type": "constituent_profile", "relationship_type": "many_to_one", "forward_label": "Funder", "reverse_label": "Grants", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 }
    ])
}

/// The spec's four business rules include one skipped for the reason
/// this module's own doc comment already documents:
/// - "Capacity rule" (event registrations >= capacity sets Full or
///   blocks further registrations) needs counting every registration
///   linked to the same Event and comparing the count to `capacity` -
///   the cross-record aggregate gap, same as Recruitment's "Job filled".
///
/// "Membership activation" and "Donation validation" both drop their
/// relationship-existence half (member/plan required; constituent
/// required) for the reason this module's own doc comment already
/// documents; "Donation validation"'s amount check needs no stage gate
/// at all - it's a standalone condition that applies on every save,
/// same as the constraint that condition IS the check, not a filter for
/// a separate action. "Renewal integrity" is the second rule in this
/// module (after Property Management's lease dates) to use field-to-field
/// comparison (`compare_field_key`) instead of a literal value.
fn nonprofit_association_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "membership",
            "name": "Membership activation",
            "description": "An active membership must record its start and end dates.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "membership_stage", "operator": "equals", "value": "Active" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "start_date", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "end_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "donation",
            "name": "Donation validation",
            "description": "A donation must record more than zero.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "amount", "operator": "less_than", "value": "0.01" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Enter a donation amount greater than zero." }
            ]
        },
        {
            "entity_type": "membership",
            "name": "Renewal integrity",
            "description": "A membership's renewal date cannot be earlier than its start date.",
            "match_type": "all",
            "priority": 1,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "renewal_date", "operator": "on_or_before", "value": "", "compare_field_source": "custom", "compare_field_key": "start_date" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Renewal date cannot be earlier than the start date." }
            ]
        },
        {
            "entity_type": "grant",
            "name": "Grant award",
            "description": "An awarded grant must record its awarded amount and award date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "grant_status", "operator": "equals", "value": "Awarded" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "awarded_amount", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "award_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// The spec's four workflows include two skipped for reasons this
/// module's own doc comment already documents - both "Membership
/// renewal" (30 days before end) and "Membership expired" (end date
/// passed) need a `date_reached`/`due_overdue`-style trigger on a custom
/// object's own date field, which the engine doesn't support. "Donation
/// received"'s "update campaign totals/reporting" half needs an
/// accumulating write across every donation linked to the same
/// Campaign, the same gap Construction & Contractors' "Change approved"
/// workflow hit - only its "create acknowledgement task" half is
/// automated below. "Program participation"'s "if template configured"
/// qualifier has no template system to check against, so the task is
/// created unconditionally on every new participant.
fn nonprofit_association_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "donation",
            "name": "Donation received",
            "description": "A completed donation gets an acknowledgement task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "donation_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "donation_status", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Send donation acknowledgement\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "program_participation",
            "name": "Program participation",
            "description": "A newly registered participant gets an onboarding task.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Onboarding checklist\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "grant",
            "name": "Grant application submitted",
            "description": "Logging a new grant application opens a follow-up task.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up on grant application\",\"description\":null,\"due_in_days\":14,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

/// `lanesra.auto_service` v1.0.0 - the tenth and final reference package
/// this module ships, sequenced right after Nonprofit & Association per
/// the dev spec. See this module's own doc comment for what's included
/// and what's deliberately left out.
///
/// Key-collision notes (see this module's own doc comment on the
/// Construction/Professional Services "Project" collision for the first
/// instance of this): the spec's "Appointment" object would collide with
/// Field Service's own `appointment` custom object key... actually with
/// Practice Administration's `appointment` key - renamed to
/// `vehicle_appointment`. The spec's "Inspection" object would collide
/// with Construction & Contractors' own `inspection` key - renamed to
/// `repair_inspection`.
///
/// "Customer" (spec: "Customer / Contact / Product / Service / Quote /
/// Invoice / Task (Core)") is represented the same way Practice
/// Administration's Patient and Real Estate's Client are - a direct link
/// to the built-in Contact entity, not Company, since a vehicle owner is
/// an individual, not a business (Field Service's B2B "Customer" is the
/// one exception that links to Company instead).
pub fn auto_service_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.auto_service",
        "name": "Auto Repair & Service Garage",
        "industry": "Automotive",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "vehicle", "singular_label": "Vehicle", "plural_label": "Vehicles", "icon": "🚗", "prefix": "VEH", "digits": 5 },
            { "key": "repair_order", "singular_label": "Repair Order", "plural_label": "Repair Orders", "icon": "🧾", "prefix": "RO", "digits": 5 },
            { "key": "repair_line", "singular_label": "Repair Line", "plural_label": "Repair Lines", "icon": "📄", "prefix": "RL", "digits": 5 },
            { "key": "repair_inspection", "singular_label": "Inspection", "plural_label": "Inspections", "icon": "🔎", "prefix": "INSP", "digits": 4 },
            { "key": "service_recommendation", "singular_label": "Service Recommendation", "plural_label": "Service Recommendations", "icon": "💡", "prefix": "REC", "digits": 4 },
            { "key": "vehicle_appointment", "singular_label": "Appointment", "plural_label": "Appointments", "icon": "📅", "prefix": "APT", "digits": 5 },
            { "key": "parts_order", "singular_label": "Parts Order", "plural_label": "Parts Orders", "icon": "📦", "prefix": "PO", "digits": 5 }
        ],
        "fields": auto_service_fields(),
        "relationships": auto_service_relationships(),
        "business_rules": auto_service_business_rules(),
        "workflows": auto_service_workflows(),
        "screen_layouts": [
            {
                "entity_type": "vehicle",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "details",
                            "title": "Details",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["make", "model", "year", "vin", "plate", "odometer", "vehicle_stage"] }
                            ],
                            // Indices into `relationships` below: Repair
                            // Orders (2), Appointments (1), Service
                            // Recommendations (5).
                            "related": ["2", "1", "5"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Repair Orders by Stage", "entity_type": "repair_order", "group_by_source": "custom", "group_by_field": "ro_stage", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Service Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Auto Repair & Service Garage",
            "icon": "🚗",
            "description": "Vehicles, repair orders, inspections and service recommendations for independent garages and small automotive service centers.",
            "object_keys": [
                "vehicle", "repair_order", "repair_line", "repair_inspection", "service_recommendation", "vehicle_appointment", "parts_order",
                "Contact", "Product", "Quote", "Invoice", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // Spec role names (Shop Admin, Service Advisor, Technician,
            // Shop Manager, Billing) mapped onto this build's actual role
            // set - see field_service_manifest_json's own note on this
            // same mapping.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        "seed_data": []
    })
    .to_string()
}

/// Split into one `json!` array per object, same as every other package
/// in this module, purely to stay under the `json!` macro's recursion
/// limit.
fn auto_service_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        vehicle_fields(),
        repair_order_fields(),
        repair_line_fields(),
        repair_inspection_fields(),
        service_recommendation_fields(),
        vehicle_appointment_fields(),
        parts_order_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn vehicle_fields() -> serde_json::Value {
    json!([
        { "key": "make", "entity_type": "vehicle", "label": "Make", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "model", "entity_type": "vehicle", "label": "Model", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "year", "entity_type": "vehicle", "label": "Year", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "1900", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Optional but unique when provided (spec: "VIN optional but
        // unique if populated") - same shape as Asset's serial_number and
        // Provider Profile's license_reference.
        { "key": "vin", "entity_type": "vehicle", "label": "VIN", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": true, "help_text": null, "placeholder": null },
        { "key": "plate", "entity_type": "vehicle", "label": "License Plate", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "odometer", "entity_type": "vehicle", "label": "Odometer", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // The custom-status workaround this module's own doc comment
        // documents - a Vehicle's built-in status is always Active/
        // Inactive/Archived, so the spec's own Active/Sold-Transferred/
        // Inactive vocabulary lives here instead.
        { "key": "vehicle_stage", "entity_type": "vehicle", "label": "Stage", "field_type": "select", "options": ["Active", "Sold or Transferred", "Inactive"], "required": true, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn repair_order_fields() -> serde_json::Value {
    json!([
        { "key": "ro_stage", "entity_type": "repair_order", "label": "Stage", "field_type": "select", "options": ["Draft", "Authorized", "In Progress", "Waiting Parts", "Ready", "Completed", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "complaint", "entity_type": "repair_order", "label": "Complaint", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "date_in", "entity_type": "repair_order", "label": "Date In", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "odometer_in", "entity_type": "repair_order", "label": "Odometer In", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Must not be less than odometer_in by the "Odometer validation"
        // business rule below.
        { "key": "odometer_out", "entity_type": "repair_order", "label": "Odometer Out", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Completed by the "Repair completion" business
        // rule below.
        { "key": "completion_date", "entity_type": "repair_order", "label": "Completion Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn repair_line_fields() -> serde_json::Value {
    json!([
        { "key": "line_type", "entity_type": "repair_line", "label": "Type", "field_type": "select", "options": ["Labor", "Part", "Service"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Labor", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "line_description", "entity_type": "repair_line", "label": "Description", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "quantity", "entity_type": "repair_line", "label": "Qty", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "1", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Authorized by the "Authorization" business rule
        // below.
        { "key": "price", "entity_type": "repair_line", "label": "Price", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "line_stage", "entity_type": "repair_line", "label": "Stage", "field_type": "select", "options": ["Proposed", "Authorized", "In Progress", "Complete", "Declined"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Proposed", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billable", "entity_type": "repair_line", "label": "Billable", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "true", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn repair_inspection_fields() -> serde_json::Value {
    json!([
        { "key": "checklist_type", "entity_type": "repair_inspection", "label": "Checklist Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "inspection_outcome", "entity_type": "repair_inspection", "label": "Outcome", "field_type": "select", "options": ["Pass", "Attention", "Fail"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "recommendations", "entity_type": "repair_inspection", "label": "Recommendations", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_recommendation_fields() -> serde_json::Value {
    json!([
        // Compared against a scheduled follow-up by the spec's
        // "Recommendation due" workflow - left unenforced, see this
        // module's own doc comment on date_reached-style triggers on a
        // custom object's own field.
        { "key": "recommended_date", "entity_type": "service_recommendation", "label": "Recommended Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "recommended_odometer", "entity_type": "service_recommendation", "label": "Recommended Odometer", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "recommendation_priority", "entity_type": "service_recommendation", "label": "Priority", "field_type": "select", "options": ["Low", "Medium", "High"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Medium", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "recommendation_status", "entity_type": "service_recommendation", "label": "Status", "field_type": "select", "options": ["Open", "Scheduled", "Completed", "Declined", "Deferred"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Open", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn vehicle_appointment_fields() -> serde_json::Value {
    json!([
        { "key": "appt_date", "entity_type": "vehicle_appointment", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // No time-of-day field type - see this module's own doc comment
        // (same approximation Practice Administration's Appointment uses).
        { "key": "appt_time", "entity_type": "vehicle_appointment", "label": "Time", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "service_reason", "entity_type": "vehicle_appointment", "label": "Service Reason", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "appt_stage", "entity_type": "vehicle_appointment", "label": "Status", "field_type": "select", "options": ["Requested", "Confirmed", "Checked In", "Completed", "No Show", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Requested", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn parts_order_fields() -> serde_json::Value {
    json!([
        { "key": "order_status", "entity_type": "parts_order", "label": "Status", "field_type": "select", "options": ["Ordered", "Backordered", "Received", "Cancelled"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Ordered", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "part_name", "entity_type": "parts_order", "label": "Part Name", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "vendor_reference", "entity_type": "parts_order", "label": "Vendor / Reference", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "ordered_date", "entity_type": "parts_order", "label": "Ordered Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Received by the "Parts order receipt" business rule below.
        { "key": "received_date", "entity_type": "parts_order", "label": "Received Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: the screen layout's `related` list and
/// the workflows below reference these relationships by their position
/// in this array. Relationships 7 and 8 follow the "core entity as the
/// many/optional side" pattern this module's own doc comment already
/// established (Construction's Invoice->project, Professional Services'
/// Invoice->engagement, Legal Practice's Invoice->matter, Nonprofit's
/// Invoice->membership) for the spec's "Repair Order 0:1 optional Quote/
/// Invoice". Relationship 9 doesn't come from the spec's own relationship
/// model at all - it's added purely so the "Appointment check-in" and
/// "Repair completed" workflows below have something to create/update
/// through, the same "add a relationship purely to support a workflow
/// action" pattern used for Construction's Invoice link, Professional
/// Services' Invoice link, Recruitment's Placement<->Offer and Real
/// Estate's Transaction<->Listing.
fn auto_service_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "vehicle", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Owner", "reverse_label": "Vehicles", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "vehicle_appointment", "target_entity_type": "vehicle", "relationship_type": "many_to_one", "forward_label": "Vehicle", "reverse_label": "Appointments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "repair_order", "target_entity_type": "vehicle", "relationship_type": "many_to_one", "forward_label": "Vehicle", "reverse_label": "Repair Orders", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "repair_line", "target_entity_type": "repair_order", "relationship_type": "many_to_one", "forward_label": "Repair Order", "reverse_label": "Lines", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "repair_inspection", "target_entity_type": "repair_order", "relationship_type": "many_to_one", "forward_label": "Repair Order", "reverse_label": "Inspections", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "service_recommendation", "target_entity_type": "vehicle", "relationship_type": "many_to_one", "forward_label": "Vehicle", "reverse_label": "Service Recommendations", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 6 */ { "source_entity_type": "repair_line", "target_entity_type": "Product", "relationship_type": "many_to_one", "forward_label": "Product / Service", "reverse_label": "Repair Lines", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 7 */ { "source_entity_type": "Quote", "target_entity_type": "repair_order", "relationship_type": "many_to_one", "forward_label": "Repair Order", "reverse_label": "Quotes", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 2 },
        /* 8 */ { "source_entity_type": "Invoice", "target_entity_type": "repair_order", "relationship_type": "many_to_one", "forward_label": "Repair Order", "reverse_label": "Invoices", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 9 */ { "source_entity_type": "repair_order", "target_entity_type": "vehicle_appointment", "relationship_type": "many_to_one", "forward_label": "Originating Appointment", "reverse_label": "Repair Orders", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 4 },
        /* 10 */ { "source_entity_type": "parts_order", "target_entity_type": "repair_order", "relationship_type": "many_to_one", "forward_label": "Repair Order", "reverse_label": "Parts Orders", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 5 }
    ])
}

/// Of the spec's four business rules, "Vehicle identity" is dropped
/// entirely as a rule - its "require owner/customer" half needs a rule
/// that can require a relationship link exists (this module's own doc
/// comment's gap, same as Practice Administration's "Patient identity"),
/// and its "require make/model" half is instead enforced the same way
/// every other package enforces an always-required field: `required:
/// true` on the field definition itself (see `vehicle_fields`), needing
/// no rule at all. "Authorization"'s "unless zero-price permission"
/// exception is dropped too - there's no permission-aware condition -
/// leaving a plain "Authorized requires a price" check. "Odometer
/// validation" is this module's first business rule to use field-to-field
/// comparison on a *numeric* pair (`less_than` is correct here, unlike
/// Nonprofit's date-pair "Renewal integrity", which needed
/// `on_or_before` instead - see that rule's own history in this module).
fn auto_service_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "repair_order",
            "name": "Repair completion",
            "description": "A completed repair order must record its completion date and odometer out reading.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "ro_stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "completion_date", "target_field_source": "custom", "action_value": null, "message": null },
                { "action_type": "require", "target_field_key": "odometer_out", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "repair_line",
            "name": "Authorization",
            "description": "An authorized repair line must record a price.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "line_stage", "operator": "equals", "value": "Authorized" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "price", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "repair_order",
            "name": "Odometer validation",
            "description": "Odometer out cannot be less than odometer in.",
            "match_type": "all",
            "priority": 1,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "odometer_out", "operator": "less_than", "value": "", "compare_field_source": "custom", "compare_field_key": "odometer_in" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Odometer out cannot be less than odometer in." }
            ]
        },
        {
            "entity_type": "parts_order",
            "name": "Parts order receipt",
            "description": "A received parts order must record its received date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "order_status", "operator": "equals", "value": "Received" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "received_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// Of the spec's five workflows, "Recommendation due" is dropped entirely
/// for the reason this module's own doc comment already documents - it
/// needs a `date_reached`-style trigger on a custom object's own date/
/// number field. "Appointment check-in"'s "copy customer/vehicle
/// [context]" half is dropped for the newly-noted `create_record`
/// name_template gap this module's own doc comment now documents - only
/// the create-and-link half runs. "Repair authorized"'s "if no line
/// assignment" qualifier has no per-line-assignment check to run (a
/// cross-record read), so the technician assignment task is created
/// unconditionally, same as Nonprofit's "Program participation" workflow
/// dropping its own template qualifier for an analogous reason. "Repair
/// completed"'s "update Vehicle service history" half is dropped for the
/// accumulating-write gap (same as Construction's "Change approved"),
/// and its "create/update draft Invoice" half is dropped too -
/// `create_record`'s `is_creatable_entity_type` check only allows
/// Company and active custom objects (every other core entity needs a
/// required relational or line-item field a no-code action can't safely
/// synthesize, per that function's own doc comment), so Invoice can't be
/// created this way at all; the Invoice->repair_order relationship above
/// still lets someone create one by hand and link it through the normal
/// related-list flow. Only "close appointment" runs, via
/// `update_related_record` against relationship 9 - a repair order
/// created some other way (not through check-in) simply has nothing
/// linked there, so that action is a no-op for it.
fn auto_service_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "vehicle_appointment",
            "name": "Appointment check-in",
            "description": "Checking in an appointment opens a draft repair order for it.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "appt_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "appt_stage", "operator": "equals", "value": "Checked In" }
            ],
            "actions": [
                { "action_type": "create_record", "params_json": "{\"entity_type\":\"repair_order\",\"relationship_ref\":9,\"name_template\":null}" }
            ]
        },
        {
            "entity_type": "repair_order",
            "name": "Repair authorized",
            "description": "Authorizing a repair order creates a technician assignment task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "ro_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "ro_stage", "operator": "equals", "value": "Authorized" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Assign technician\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "repair_order",
            "name": "Repair completed",
            "description": "Completing a repair order closes any originating appointment.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "ro_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "ro_stage", "operator": "equals", "value": "Completed" }
            ],
            "actions": [
                { "action_type": "update_related_record", "params_json": "{\"relationship_ref\":9,\"target_field_key\":\"appt_stage\",\"target_field_source\":\"custom\",\"value\":\"Completed\",\"copy_from_field_key\":null}" }
            ]
        },
        {
            "entity_type": "vehicle_appointment",
            "name": "No show",
            "description": "A no-show appointment gets a reschedule task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "appt_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "appt_stage", "operator": "equals", "value": "No Show" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Reschedule appointment\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "parts_order",
            "name": "Parts backordered",
            "description": "A backordered part gets a follow-up task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "order_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "order_status", "operator": "equals", "value": "Backordered" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Follow up on backordered part\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.policy_admin_claims -----------------------------------------

/// `lanesra.policy_admin_claims` v1.0.0 - an eleventh reference package,
/// added after the original ten-vertical spec on direct product request:
/// insurance carriers, MGAs and TPAs running policy administration and
/// claims handling on the same extensible object model as everything
/// else in this module.
///
/// Object model and vocabulary are checked against how real policy admin
/// (PAS) and claims systems actually shape this data - Guidewire
/// PolicyCenter/ClaimCenter, Duck Creek, and the ACORD data standards -
/// rather than invented from scratch:
/// - `policy` / `coverage` / `insured_risk` mirrors a PAS's own Policy ->
///   Coverage (a.k.a. "Coverage Part") -> insured-item structure; a
///   `coverage` can optionally point at one specific `insured_risk`
///   (e.g. Collision on a named vehicle) or apply at the policy level
///   (e.g. Homeowners liability), matching real multi-vehicle/
///   multi-location policies rather than assuming exactly one insured
///   item per policy.
/// - `endorsement` is the PAS term of art for a mid-term policy change;
///   kept as its own object the same way every other package here keeps
///   a document/change-order object separate from what it amends
///   (Construction & Contractors' `change_order`, Auto Repair's
///   `parts_order`).
/// - `claim` / `claimant` / `loss_reserve` / `claim_payment` /
///   `adjuster_assignment` / `subrogation` mirrors ClaimCenter's own
///   claim-file shape: one `claim` against a `policy`, one or more
///   `claimant`s against it (first- or third-party), reserves tracked
///   separately from payments (real claims reserving keeps Indemnity/
///   Expense/Legal reserves distinct from what's actually been paid out -
///   exactly what `loss_reserve` vs. `claim_payment` models), one or more
///   adjusters assigned, and an optional subrogation case when a third
///   party is responsible for the loss.
///
/// Simplifications, beyond the standard omissions this module's own doc
/// comment already covers:
/// - The Named Insured is a single `Contact`, not a polymorphic person-or-
///   organization link - real PAS systems support commercial policies
///   insuring an organization too, but a manifest relationship always
///   targets one fixed entity type. Personal lines (the more common case
///   for a `Contact`-shaped CRM record) is the one modeled; a commercial-
///   lines variant relating `policy` to `Company` instead would need its
///   own package, the same reasoning `professional_services_manifest_json`
///   gives for not reusing Construction's `project` object key.
/// - No separate Underwriting Referral object - underwriting is a status
///   on `policy` itself (`underwriting_status`), not a queue of its own
///   records, since nothing else in this model needs to relate to one
///   individual underwriting decision the way, say, a `claim` needs its
///   own `loss_reserve` history.
/// - Reserve and payment amounts are plain fields an adjuster sets, not a
///   rollup automatically summed from anything - this engine has no
///   rollup/aggregate field type, the same reason every other package's
///   totals (Property Management's `rent_amount`, Construction's premium
///   fields) are plain entries too.
/// - `claim_payment`'s payee is a plain select field (`payee_type`), not a
///   relationship to whichever record was actually paid - the payee can
///   be the insured, a claimant, a repair shop, or an attorney, and this
///   engine's relationships always target one fixed entity type, the
///   same reason `policy`'s own Named Insured is Contact-only above.
pub fn policy_admin_claims_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.policy_admin_claims",
        "name": "Policy Administration & Claims Management",
        "industry": "Insurance",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "policy", "singular_label": "Policy", "plural_label": "Policies", "icon": "📋", "prefix": "POL", "digits": 6 },
            { "key": "coverage", "singular_label": "Coverage", "plural_label": "Coverages", "icon": "🛡", "prefix": "COV", "digits": 5 },
            { "key": "insured_risk", "singular_label": "Insured Risk", "plural_label": "Insured Risks", "icon": "📦", "prefix": "RISK", "digits": 5 },
            { "key": "endorsement", "singular_label": "Endorsement", "plural_label": "Endorsements", "icon": "✏", "prefix": "END", "digits": 5 },
            { "key": "claim", "singular_label": "Claim", "plural_label": "Claims", "icon": "🧾", "prefix": "CLM", "digits": 6 },
            { "key": "claimant", "singular_label": "Claimant", "plural_label": "Claimants", "icon": "🧑", "prefix": "CLT", "digits": 5 },
            { "key": "loss_reserve", "singular_label": "Loss Reserve", "plural_label": "Loss Reserves", "icon": "💰", "prefix": "RES", "digits": 6 },
            { "key": "claim_payment", "singular_label": "Claim Payment", "plural_label": "Claim Payments", "icon": "💳", "prefix": "PAY", "digits": 6 },
            { "key": "adjuster_assignment", "singular_label": "Adjuster Assignment", "plural_label": "Adjuster Assignments", "icon": "🕵", "prefix": "ADJ", "digits": 5 },
            { "key": "subrogation", "singular_label": "Subrogation Case", "plural_label": "Subrogation Cases", "icon": "🔁", "prefix": "SUB", "digits": 5 }
        ],
        "fields": policy_admin_claims_fields(),
        "relationships": policy_admin_claims_relationships(),
        "business_rules": policy_admin_claims_business_rules(),
        "workflows": policy_admin_claims_workflows(),
        "screen_layouts": [
            {
                "entity_type": "policy",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "overview",
                            "title": "Overview",
                            "sections": [
                                { "id": "details", "title": "Details", "columns": 2, "fields": ["policy_stage", "product_line", "policy_type", "effective_date", "expiration_date", "total_premium", "underwriting_status", "billing_plan"] }
                            ],
                            // Indices into `relationships` below: Coverages
                            // (1), Insured Risks (2), Endorsements (4),
                            // Claims (5).
                            "related": ["1", "2", "4", "5"]
                        }
                    ]
                },
                "publish": true
            },
            {
                "entity_type": "claim",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "loss",
                            "title": "Loss Information",
                            "sections": [
                                { "id": "overview", "title": "Overview", "columns": 2, "fields": ["claim_status", "date_of_loss", "date_reported", "loss_type", "reserve_amount", "paid_amount", "fraud_referral", "litigation_flag"] }
                            ],
                            // Indices into `relationships` below: Claimants
                            // (7), Loss Reserves (8), Payments (9), Adjuster
                            // Assignments (10), Subrogation (12).
                            "related": ["7", "8", "9", "10", "12"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Policies by Status", "entity_type": "policy", "group_by_source": "custom", "group_by_field": "policy_stage", "aggregate": "count", "sum_field_key": null },
            { "name": "Claims by Status", "entity_type": "claim", "group_by_source": "custom", "group_by_field": "claim_status", "aggregate": "count", "sum_field_key": null },
            { "name": "Claims by Loss Type", "entity_type": "claim", "group_by_source": "custom", "group_by_field": "loss_type", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Policy Admin & Claims Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } },
                { "kind": "chart", "config": { "report_ref": 1, "chart_type": "bar" } },
                { "kind": "chart", "config": { "report_ref": 2, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Policy Administration & Claims Management",
            "icon": "🛡",
            "description": "Policies, coverages, insured risks and endorsements on one side; claims, claimants, reserves, payments, adjuster assignments and subrogation on the other - the full policy admin and claims lifecycle for a carrier, MGA or TPA.",
            "object_keys": [
                "policy", "coverage", "insured_risk", "endorsement", "claim", "claimant",
                "loss_reserve", "claim_payment", "adjuster_assignment", "subrogation", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            // See field_service_manifest_json's own note on mapping the spec's role
            // names onto this build's actual role set.
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "editor" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object in this data model - every object
        // here is a live business record - so nothing to seed.
        "seed_data": []
    })
    .to_string()
}

fn policy_admin_claims_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        policy_fields(),
        coverage_fields(),
        insured_risk_fields(),
        endorsement_fields(),
        claim_fields(),
        claimant_fields(),
        loss_reserve_fields(),
        claim_payment_fields(),
        adjuster_assignment_fields(),
        subrogation_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn policy_fields() -> serde_json::Value {
    json!([
        { "key": "product_line", "entity_type": "policy", "label": "Product Line", "field_type": "select", "options": ["Personal Auto", "Homeowners", "Commercial Property", "Commercial General Liability", "Workers Compensation", "Umbrella/Excess", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "policy_type", "entity_type": "policy", "label": "Policy Type", "field_type": "select", "options": ["New Business", "Renewal", "Rewrite"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "New Business", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "effective_date", "entity_type": "policy", "label": "Effective Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expiration_date", "entity_type": "policy", "label": "Expiration Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "term", "entity_type": "policy", "label": "Term", "field_type": "select", "options": ["Month-to-Month", "Semi-Annual", "Annual", "Multi-Year"], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "Annual", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "billing_plan", "entity_type": "policy", "label": "Billing Plan", "field_type": "select", "options": ["Annual", "Semi-Annual", "Quarterly", "Monthly"], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "Monthly", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "total_premium", "entity_type": "policy", "label": "Total Premium", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "underwriting_status", "entity_type": "policy", "label": "Underwriting Status", "field_type": "select", "options": ["Pending Review", "Approved", "Declined", "Referred"], "required": false, "show_in_list": false, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Pending Review", "is_unique": false, "help_text": null, "placeholder": null },
        // Named policy_stage, not status - every custom object already has a
        // fixed built-in Active/Inactive/Archived status (see this module's
        // own doc comment); a same-named custom field would collide with it.
        // Drives the "Policy bound" workflow below.
        { "key": "policy_stage", "entity_type": "policy", "label": "Status", "field_type": "select", "options": ["Quote", "Bound", "In Force", "Renewed", "Non-Renewed", "Cancelled", "Expired", "Lapsed"], "required": true, "show_in_list": true, "sort_order": 8, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Quote", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "cancellation_reason", "entity_type": "policy", "label": "Cancellation Reason", "field_type": "select", "options": ["Non-Payment", "Underwriting", "Insured Request", "Other"], "required": false, "show_in_list": false, "sort_order": 9, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn coverage_fields() -> serde_json::Value {
    json!([
        { "key": "coverage_type", "entity_type": "coverage", "label": "Coverage Type", "field_type": "select", "options": ["Bodily Injury Liability", "Property Damage Liability", "Collision", "Comprehensive", "Uninsured/Underinsured Motorist", "Medical Payments", "Dwelling", "Other Structures", "Personal Property", "Loss of Use", "General Liability", "Property", "Workers Compensation", "Umbrella", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "limit_amount", "entity_type": "coverage", "label": "Limit", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "deductible_amount", "entity_type": "coverage", "label": "Deductible", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "premium_amount", "entity_type": "coverage", "label": "Premium", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "coverage_status", "entity_type": "coverage", "label": "Status", "field_type": "select", "options": ["Active", "Pending", "Removed"], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn insured_risk_fields() -> serde_json::Value {
    json!([
        { "key": "risk_type", "entity_type": "insured_risk", "label": "Risk Type", "field_type": "select", "options": ["Vehicle", "Real Property", "Location", "Equipment/Contents", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description", "entity_type": "insured_risk", "label": "Description", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Generic identifier - a VIN for a vehicle, a parcel/serial number
        // for property or equipment.
        { "key": "identifier", "entity_type": "insured_risk", "label": "Identifier", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "VIN, parcel number, serial number, or other identifying number for this risk.", "placeholder": null },
        { "key": "insured_value", "entity_type": "insured_risk", "label": "Insured Value", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "risk_status", "entity_type": "insured_risk", "label": "Status", "field_type": "select", "options": ["Active", "Removed"], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn endorsement_fields() -> serde_json::Value {
    json!([
        { "key": "endorsement_type", "entity_type": "endorsement", "label": "Endorsement Type", "field_type": "select", "options": ["Add Coverage", "Remove Coverage", "Change Limit or Deductible", "Add Insured Risk", "Remove Insured Risk", "Address/Name Change", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "effective_date", "entity_type": "endorsement", "label": "Effective Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "premium_change_amount", "entity_type": "endorsement", "label": "Premium Change", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": "Positive to add premium, negative to return premium.", "placeholder": null },
        { "key": "description", "entity_type": "endorsement", "label": "Description", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Endorsement processed" workflow below.
        { "key": "endorsement_status", "entity_type": "endorsement", "label": "Status", "field_type": "select", "options": ["Draft", "Pending Approval", "Approved", "Processed", "Rejected"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn claim_fields() -> serde_json::Value {
    json!([
        { "key": "date_of_loss", "entity_type": "claim", "label": "Date of Loss", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "date_reported", "entity_type": "claim", "label": "Date Reported", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "loss_type", "entity_type": "claim", "label": "Loss Type", "field_type": "select", "options": ["Collision", "Fire", "Theft", "Water Damage", "Wind/Hail", "Liability - Bodily Injury", "Liability - Property Damage", "Vandalism", "Weather/Catastrophe", "Workers Comp Injury", "Other"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "description_of_loss", "entity_type": "claim", "label": "Description of Loss", "field_type": "text", "options": [], "required": true, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Claim closure follow-up" workflow and the "Claim
        // payout documentation" business rule below.
        { "key": "claim_status", "entity_type": "claim", "label": "Status", "field_type": "select", "options": ["New", "Under Investigation", "Pending Documentation", "Approved", "Denied", "In Litigation", "Closed - Paid", "Closed - Denied", "Reopened"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "New", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reserve_amount", "entity_type": "claim", "label": "Total Reserve", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": "The claim's own headline reserve figure - see the Loss Reserves related list for the itemized Indemnity/Expense/Legal breakdown.", "placeholder": null },
        // Required-when-Closed-Paid by the "Claim payout documentation" rule below.
        { "key": "paid_amount", "entity_type": "claim", "label": "Total Paid", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "SIU referral escalation" workflow below.
        { "key": "fraud_referral", "entity_type": "claim", "label": "Fraud Referral", "field_type": "select", "options": ["No", "Under Review", "Confirmed - SIU"], "required": false, "show_in_list": false, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "No", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "litigation_flag", "entity_type": "claim", "label": "Litigation", "field_type": "select", "options": ["No", "Yes - Pending", "Yes - Resolved"], "required": false, "show_in_list": false, "sort_order": 8, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "No", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn claimant_fields() -> serde_json::Value {
    json!([
        { "key": "claimant_type", "entity_type": "claimant", "label": "Claimant Type", "field_type": "select", "options": ["First Party - Named Insured", "First Party - Additional Insured", "Third Party"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "relationship_to_insured", "entity_type": "claimant", "label": "Relationship to Insured", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "injury_involved", "entity_type": "claimant", "label": "Injury Involved", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Attorney representation" business rule below.
        { "key": "represented_by_attorney", "entity_type": "claimant", "label": "Represented by Attorney", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "attorney_name", "entity_type": "claimant", "label": "Attorney Name", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn loss_reserve_fields() -> serde_json::Value {
    json!([
        { "key": "reserve_type", "entity_type": "loss_reserve", "label": "Reserve Type", "field_type": "select", "options": ["Indemnity", "Expense", "Legal/Defense"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reserve_amount", "entity_type": "loss_reserve", "label": "Reserve Amount", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reserve_date", "entity_type": "loss_reserve", "label": "Reserve Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "reserve_reason", "entity_type": "loss_reserve", "label": "Reason", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn claim_payment_fields() -> serde_json::Value {
    json!([
        { "key": "payment_type", "entity_type": "claim_payment", "label": "Payment Type", "field_type": "select", "options": ["Indemnity", "Expense", "Deductible Recovery", "Subrogation Recovery", "Salvage Recovery"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "payee_type", "entity_type": "claim_payment", "label": "Payee", "field_type": "select", "options": ["Insured", "Claimant", "Third Party", "Vendor/Repair Shop", "Attorney"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "payment_amount", "entity_type": "claim_payment", "label": "Payment Amount", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "payment_date", "entity_type": "claim_payment", "label": "Payment Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "payment_status", "entity_type": "claim_payment", "label": "Status", "field_type": "select", "options": ["Issued", "Voided", "Reissued", "Stopped"], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Issued", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "check_number", "entity_type": "claim_payment", "label": "Check/Reference Number", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn adjuster_assignment_fields() -> serde_json::Value {
    json!([
        { "key": "adjuster_role", "entity_type": "adjuster_assignment", "label": "Adjuster Role", "field_type": "select", "options": ["Desk Adjuster", "Field Adjuster", "Independent Adjuster", "SIU Investigator", "Appraiser"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "assigned_date", "entity_type": "adjuster_assignment", "label": "Assigned Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "assignment_status", "entity_type": "adjuster_assignment", "label": "Status", "field_type": "select", "options": ["Assigned", "Active", "Completed", "Reassigned"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Assigned", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "notes", "entity_type": "adjuster_assignment", "label": "Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn subrogation_fields() -> serde_json::Value {
    json!([
        // Drives the "Subrogation recovery documentation" business rule below.
        { "key": "subrogation_status", "entity_type": "subrogation", "label": "Status", "field_type": "select", "options": ["Identified", "Demand Sent", "In Negotiation", "Recovered", "Closed - No Recovery"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Identified", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "demand_amount", "entity_type": "subrogation", "label": "Demand Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Recovered by the "Subrogation recovery documentation" rule below.
        { "key": "recovered_amount", "entity_type": "subrogation", "label": "Recovered Amount", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "responsible_party_description", "entity_type": "subrogation", "label": "Responsible Party", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: `screen_layouts` above and every test
/// in `policy_admin_claims_reference_package.rs` that resolves a
/// relationship by source/target pair reference these by position.
fn policy_admin_claims_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "policy", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Named Insured", "reverse_label": "Policies", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "coverage", "target_entity_type": "policy", "relationship_type": "many_to_one", "forward_label": "Policy", "reverse_label": "Coverages", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 2 */ { "source_entity_type": "insured_risk", "target_entity_type": "policy", "relationship_type": "many_to_one", "forward_label": "Policy", "reverse_label": "Insured Risks", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 1 },
        /* 3 */ { "source_entity_type": "coverage", "target_entity_type": "insured_risk", "relationship_type": "many_to_one", "forward_label": "Insured Risk", "reverse_label": "Coverages", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "endorsement", "target_entity_type": "policy", "relationship_type": "many_to_one", "forward_label": "Policy", "reverse_label": "Endorsements", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 2 },
        /* 5 */ { "source_entity_type": "claim", "target_entity_type": "policy", "relationship_type": "many_to_one", "forward_label": "Policy", "reverse_label": "Claims", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 3 },
        /* 6 */ { "source_entity_type": "claim", "target_entity_type": "insured_risk", "relationship_type": "many_to_one", "forward_label": "Insured Risk", "reverse_label": "Claims", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 7 */ { "source_entity_type": "claimant", "target_entity_type": "claim", "relationship_type": "many_to_one", "forward_label": "Claim", "reverse_label": "Claimants", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 8 */ { "source_entity_type": "loss_reserve", "target_entity_type": "claim", "relationship_type": "many_to_one", "forward_label": "Claim", "reverse_label": "Loss Reserves", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 1 },
        /* 9 */ { "source_entity_type": "claim_payment", "target_entity_type": "claim", "relationship_type": "many_to_one", "forward_label": "Claim", "reverse_label": "Payments", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 2 },
        /* 10 */ { "source_entity_type": "adjuster_assignment", "target_entity_type": "claim", "relationship_type": "many_to_one", "forward_label": "Claim", "reverse_label": "Adjuster Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 3 },
        /* 11 */ { "source_entity_type": "adjuster_assignment", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Adjusting Firm", "reverse_label": "Adjuster Assignments", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 12 */ { "source_entity_type": "subrogation", "target_entity_type": "claim", "relationship_type": "many_to_one", "forward_label": "Claim", "reverse_label": "Subrogation", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 4 },
        /* 13 */ { "source_entity_type": "subrogation", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Responsible Party Insurer", "reverse_label": "Subrogation Cases", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 }
    ])
}

fn policy_admin_claims_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "policy",
            "name": "Policy date validation",
            "description": "A policy's expiration date must be after its effective date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "expiration_date", "operator": "on_or_before", "value": "", "compare_field_source": "custom", "compare_field_key": "effective_date" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Policy expiration date must be after the effective date." }
            ]
        },
        {
            "entity_type": "claim",
            "name": "Claim payout documentation",
            "description": "A claim closed as paid must record the amount paid.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "equals", "value": "Closed - Paid" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "paid_amount", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "claimant",
            "name": "Attorney representation",
            "description": "A claimant represented by an attorney must record the attorney's name.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "represented_by_attorney", "operator": "equals", "value": "true" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "attorney_name", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "subrogation",
            "name": "Subrogation recovery documentation",
            "description": "A subrogation case marked Recovered must record the amount recovered.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "subrogation_status", "operator": "equals", "value": "Recovered" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "recovered_amount", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// None of these reach for `update_related_record` - unlike Property
/// Management's lease/unit pair, nothing in this object model has an
/// obvious single field on a *related* record that a claim or policy
/// event should just overwrite (a coverage's own status doesn't follow
/// mechanically from its policy being bound, the way a unit's occupancy
/// follows from its lease being active). Every workflow here creates a
/// task instead - the same fallback Property Management's own "Maintenance
/// intake" and Auto Repair's "No show"/"Parts backordered" workflows use.
fn policy_admin_claims_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "policy",
            "name": "Policy bound",
            "description": "Binding a policy creates a task to issue documents to the named insured.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "policy_stage",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "policy_stage", "operator": "equals", "value": "Bound" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Issue policy documents to the named insured\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "claim",
            "name": "Claim intake",
            "description": "A newly reported claim gets a task to assign an adjuster and begin investigation.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Assign an adjuster and begin investigation\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "claim",
            "name": "SIU referral escalation",
            "description": "A claim confirmed for SIU referral gets an escalation task.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "fraud_referral",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "fraud_referral", "operator": "equals", "value": "Confirmed - SIU" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Notify SIU and legal\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "claim",
            "name": "Claim closure follow-up",
            "description": "A closed claim (paid or denied) gets a task to send the closure letter.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "claim_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "claim_status", "operator": "in_list", "value": "Closed - Paid|Closed - Denied" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Send closure letter to claimant\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "endorsement",
            "name": "Endorsement processed",
            "description": "A processed endorsement gets a task to update billing for the premium change.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "endorsement_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "endorsement_status", "operator": "equals", "value": "Processed" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Update billing for the premium change\",\"description\":null,\"due_in_days\":1,\"assignee_user_id\":null}" }
            ]
        }
    ])
}

// --- lanesra.industry_foundation -----------------------------------------

/// `lanesra.industry_foundation` v1.0.0 - see this module's own doc comment
/// for the two new engine-limitation gaps it surfaced (self-referential
/// relationships; generic polymorphic relationship targets) and for why
/// it's different in kind from the eleven packages above it: this one is
/// shared cross-industry plumbing other packages can *depend on*, not its
/// own business vertical - the first real exercise of
/// `IndustryPackageManifest::dependencies` and of one package's objects/
/// relationships/fields targeting another already-installed package's
/// object keys, both already fully supported by `industry_package_service`
/// (confirmed by reading `validate()` and
/// `custom_object_service::is_valid_dynamic_entity_type` directly, not
/// assumed).
///
/// Modeled after this spec's own "Lanesra Industry Foundation" chapter -
/// Microsoft Common Data Model's cross-industry entities, ServiceNow's
/// foundational/operational split, Salesforce's additive industry
/// extensions around shared CRM entities - trimmed to what this engine can
/// actually deliver honestly today:
/// - `party` / `party_role` / `party_relationship`: a person-or-
///   organization abstraction, generic roles a party holds (with effective
///   dates), and party-to-party relationships (ownership %, guarantor,
///   household, ...). `party` doesn't *replace* Contact/Company - it wraps
///   an optional link to either one (a relationship can't target "Company
///   OR Contact" - every relationship has exactly one fixed target type),
///   so nothing enforces that at least one is actually set (the same "no
///   rule can require a relationship link, only a field value" gap
///   `practice_admin_manifest_json` already documents). `party_role`
///   deliberately does NOT attempt the source spec's generic "role in the
///   context of any object" (e.g. "Producer on this specific Policy") -
///   that needs a polymorphic relationship target this engine doesn't have
///   (see this module's own doc comment); a consuming package wanting a
///   role-in-context relates its own object directly to `party`/
///   `party_role` instead, the same fixed-target pattern every other
///   package in this file already uses. `party_relationship` needs two
///   separate relationships to the *same* target type (`from_party`,
///   `to_party`, both -> `party`) - confirmed working, not assumed:
///   `relationship_service::slugify` auto-suffixes the second one's key
///   rather than rejecting the collision.
/// - `organization_unit`: a department/branch/unit within a Company, with
///   effective dates. Its own natural parent-unit hierarchy can't be a
///   real relationship - self-referential relationships are rejected (see
///   this module's own doc comment) - so `parent_unit_name` is a plain,
///   unenforced text field instead of a link.
/// - `location`: site/address/geo/timezone in one object rather than a
///   separate Location + Address pair the source spec calls for - nothing
///   in that spec's own relationship table ever needs Address addressed
///   independently of its owning Location, so splitting it would only add
///   indirection with no realized benefit (the same reasoning
///   `professional_services_manifest_json` already gives for not
///   reusing Construction's `project` key elsewhere in this file).
/// - `contact_point`: kept as its own object (not folded into `party` as
///   plain fields) because a party genuinely has *many* of these (work
///   email, mobile, home phone, ...), a real one-to-many the other fields-
///   only folds in this package don't have.
/// - `external_identifier`, `interaction`, `document_record`,
///   `financial_transaction`, `data_quality_issue`: each wants to relate to
///   "whatever record this is about," a generic polymorphic relationship
///   target this engine doesn't have (see this module's own doc comment).
///   Each is instead given a plain `linked_object_type`/`linked_object_id`
///   text-field pair - stored, visible, but not a real link: no clickable
///   related list, no cascade/restrict delete behavior, nothing stopping a
///   typo in either field. Recorded once here rather than repeated per
///   object.
/// - `enterprise_asset`: named with an `enterprise_` prefix, not plain
///   `asset` - Field Service already claims that exact object key, and a
///   custom object's key is workspace-wide (see this module's own doc
///   comment on `engagement` vs. `project` for the identical reasoning).
///   No parent-asset link for the same self-referential-relationship
///   reason as `organization_unit`.
/// - `agreement`: a lighter-weight contractual master than the core
///   `Contract` entity, for commitments that don't need a full contract
///   (e.g. a producer appointment) - optionally related to a real Contract
///   when one does back it, rather than forcing every agreement through
///   the heavier object.
///
/// Deliberately left out of this package entirely, not merely trimmed:
/// - **Product Model / Product Offering** - the source spec's own Field
///   Service chapter already says to "use the generic Lanesra Product/
///   Service catalog rather than a separate parts catalog"; a second
///   "product" concept sitting next to the platform's existing built-in
///   Product entity (already the target of every package's own line-item
///   objects) would only invite confusion between two different things
///   both called a product, not add real capability.
/// - **Reference Code Set** - cannot be built as anything more than a
///   decorative, unenforced list: no field type or mechanism exists today
///   where a `select` field's options are dynamically pulled from another
///   object's rows (`CUSTOM_FIELD_TYPES` is a fixed text/number/date/
///   boolean/select set). Shipping it would misrepresent what it does;
///   real support needs a new field type, engine work, not modeling.
/// - **Automatic Data Quality Issue generation** - the source spec's own
///   vision has these auto-created by a background rules engine running
///   on every save/import/API call across every object. That's a new
///   cross-cutting validation subsystem, not a bigger manifest; this
///   package ships `data_quality_issue` as a plain object other workflows
///   or an admin can create/relate to, not the automatic detection engine
///   itself.
///
/// Not retrofitted onto the eleven packages above: none of them are
/// changed to consume these shared objects in this pass - each keeps its
/// own independent party-like/asset-like fields exactly as already
/// shipped and tested. Adopting Foundation objects is a later, per-package
/// decision, not automatic.
pub fn lanesra_industry_foundation_manifest_json() -> String {
    json!({
        "format_version": 1,
        "package_id": "lanesra.industry_foundation",
        "name": "Lanesra Industry Foundation",
        "industry": "Cross-Industry",
        "version": "1.0.0",
        "min_lanesra_version": "0.11.0",
        "dependencies": [],
        "objects": [
            { "key": "party", "singular_label": "Party", "plural_label": "Parties", "icon": "🧑‍🤝‍🧑", "prefix": "PTY", "digits": 6 },
            { "key": "party_role", "singular_label": "Party Role", "plural_label": "Party Roles", "icon": "🎭", "prefix": "PROL", "digits": 6 },
            { "key": "party_relationship", "singular_label": "Party Relationship", "plural_label": "Party Relationships", "icon": "🔗", "prefix": "PREL", "digits": 6 },
            { "key": "organization_unit", "singular_label": "Organization Unit", "plural_label": "Organization Units", "icon": "🏢", "prefix": "OU", "digits": 4 },
            { "key": "location", "singular_label": "Location", "plural_label": "Locations", "icon": "📍", "prefix": "LOC", "digits": 5 },
            { "key": "contact_point", "singular_label": "Contact Point", "plural_label": "Contact Points", "icon": "☎", "prefix": "CP", "digits": 6 },
            { "key": "external_identifier", "singular_label": "External Identifier", "plural_label": "External Identifiers", "icon": "🆔", "prefix": "EXTID", "digits": 6 },
            { "key": "consent_preference", "singular_label": "Consent / Preference", "plural_label": "Consents / Preferences", "icon": "✅", "prefix": "CONS", "digits": 6 },
            { "key": "enterprise_asset", "singular_label": "Asset", "plural_label": "Assets", "icon": "📦", "prefix": "EAST", "digits": 6 },
            { "key": "agreement", "singular_label": "Agreement", "plural_label": "Agreements", "icon": "📜", "prefix": "AGR", "digits": 5 },
            { "key": "service_case", "singular_label": "Service Case", "plural_label": "Service Cases", "icon": "🗂", "prefix": "CASE", "digits": 5 },
            { "key": "interaction", "singular_label": "Interaction", "plural_label": "Interactions", "icon": "💬", "prefix": "INTX", "digits": 6 },
            { "key": "document_record", "singular_label": "Document Record", "plural_label": "Document Records", "icon": "🗎", "prefix": "DOC", "digits": 6 },
            { "key": "financial_transaction", "singular_label": "Financial Transaction", "plural_label": "Financial Transactions", "icon": "💵", "prefix": "FTXN", "digits": 6 },
            { "key": "data_quality_issue", "singular_label": "Data Quality Issue", "plural_label": "Data Quality Issues", "icon": "⚠", "prefix": "DQI", "digits": 6 }
        ],
        "fields": lanesra_industry_foundation_fields(),
        "relationships": lanesra_industry_foundation_relationships(),
        "business_rules": lanesra_industry_foundation_business_rules(),
        "workflows": lanesra_industry_foundation_workflows(),
        "screen_layouts": [
            {
                "entity_type": "party",
                "name": "Default",
                "draft": {
                    "tabs": [
                        {
                            "id": "overview",
                            "title": "Overview",
                            "sections": [
                                { "id": "details", "title": "Details", "columns": 2, "fields": ["party_type", "display_name", "party_status"] }
                            ],
                            // Indices into `relationships` below: Roles (2), Relationships From (3),
                            // Contact Points (8), Locations (6), Agreements (12), Cases (14).
                            "related": ["2", "3", "8", "6", "12", "14"]
                        }
                    ]
                },
                "publish": true
            }
        ],
        "reports": [
            { "name": "Parties by Type", "entity_type": "party", "group_by_source": "custom", "group_by_field": "party_type", "aggregate": "count", "sum_field_key": null },
            { "name": "Data Quality Issues by Severity", "entity_type": "data_quality_issue", "group_by_source": "custom", "group_by_field": "severity", "aggregate": "count", "sum_field_key": null }
        ],
        "dashboard": {
            "name": "Industry Foundation Dashboard",
            "widgets": [
                { "kind": "chart", "config": { "report_ref": 0, "chart_type": "bar" } },
                { "kind": "chart", "config": { "report_ref": 1, "chart_type": "bar" } }
            ],
            "publish": true
        },
        "numbering_overrides": [],
        "app": {
            "name": "Lanesra Industry Foundation",
            "icon": "🏛",
            "description": "Shared cross-industry plumbing - Party, Party Role, Party Relationship, Location, Asset, Agreement and more - other industry packages can optionally depend on and relate to, instead of each reinventing its own version.",
            "object_keys": [
                "party", "party_role", "party_relationship", "organization_unit", "location",
                "contact_point", "external_identifier", "consent_preference", "enterprise_asset",
                "agreement", "service_case", "interaction", "document_record",
                "financial_transaction", "data_quality_issue", "Task"
            ],
            "use_package_dashboard": true,
            "publish": true,
            "recommended_permissions": [
                { "role": "Administrator", "level": "editor" },
                { "role": "Manager", "level": "editor" },
                { "role": "Sales", "level": "editor" },
                { "role": "Finance", "level": "viewer" },
                { "role": "ReadOnly", "level": "viewer" }
            ]
        },
        // No pure reference/lookup object in this data model to seed sample
        // rows for, same reasoning `policy_admin_claims_manifest_json`
        // already gives for its own empty seed_data.
        "seed_data": []
    })
    .to_string()
}

fn lanesra_industry_foundation_fields() -> serde_json::Value {
    let mut all = Vec::new();
    for group in [
        party_fields(),
        party_role_fields(),
        party_relationship_fields(),
        organization_unit_fields(),
        location_fields(),
        contact_point_fields(),
        external_identifier_fields(),
        consent_preference_fields(),
        enterprise_asset_fields(),
        agreement_fields(),
        service_case_fields(),
        interaction_fields(),
        document_record_fields(),
        financial_transaction_fields(),
        data_quality_issue_fields(),
    ] {
        all.extend(group.as_array().expect("each group is a json array").clone());
    }
    serde_json::Value::Array(all)
}

fn party_fields() -> serde_json::Value {
    json!([
        { "key": "party_type", "entity_type": "party", "label": "Party Type", "field_type": "select", "options": ["Person", "Organization"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "display_name", "entity_type": "party", "label": "Display Name", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Shown even before this Party is linked to a Contact or Company.", "placeholder": null },
        { "key": "party_status", "entity_type": "party", "label": "Status", "field_type": "select", "options": ["Active", "Inactive"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn party_role_fields() -> serde_json::Value {
    json!([
        // Admin-editable list, same as any select field - a package or
        // admin can widen it after install (e.g. an insurance package
        // adding "Producer" doesn't need engine changes).
        { "key": "role_type", "entity_type": "party_role", "label": "Role Type", "field_type": "select", "options": ["Tenant", "Claimant", "Producer", "Provider", "Candidate", "Owner", "Guarantor", "Beneficiary", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_from", "entity_type": "party_role", "label": "Valid From", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Optional - an ongoing role has no end date. See the "Role date
        // order" business rule below for why an empty value here is safe.
        { "key": "valid_to", "entity_type": "party_role", "label": "Valid To", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Role ended" workflow below.
        { "key": "role_status", "entity_type": "party_role", "label": "Status", "field_type": "select", "options": ["Active", "Ended"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn party_relationship_fields() -> serde_json::Value {
    json!([
        { "key": "relationship_type", "entity_type": "party_relationship", "label": "Relationship Type", "field_type": "select", "options": ["Ownership", "Guarantor", "Household", "Referral", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "ownership_percent", "entity_type": "party_relationship", "label": "Ownership %", "field_type": "number", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": "0", "max_value": "100", "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_from", "entity_type": "party_relationship", "label": "Valid From", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_to", "entity_type": "party_relationship", "label": "Valid To", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "relationship_status", "entity_type": "party_relationship", "label": "Status", "field_type": "select", "options": ["Active", "Ended"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn organization_unit_fields() -> serde_json::Value {
    json!([
        { "key": "unit_type", "entity_type": "organization_unit", "label": "Unit Type", "field_type": "select", "options": ["Department", "Branch", "Division", "Team", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "unit_code", "entity_type": "organization_unit", "label": "Unit Code", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Not a real relationship - self-referential relationships are
        // rejected (see this module's own doc comment).
        { "key": "parent_unit_name", "entity_type": "organization_unit", "label": "Parent Unit", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - this engine can't relate an object to another record of its own same type.", "placeholder": null },
        { "key": "valid_from", "entity_type": "organization_unit", "label": "Valid From", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_to", "entity_type": "organization_unit", "label": "Valid To", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "unit_status", "entity_type": "organization_unit", "label": "Status", "field_type": "select", "options": ["Active", "Inactive"], "required": true, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn location_fields() -> serde_json::Value {
    json!([
        { "key": "location_type", "entity_type": "location", "label": "Location Type", "field_type": "select", "options": ["Site", "Office", "Warehouse", "Residence", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "address_line1", "entity_type": "location", "label": "Address Line 1", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "address_line2", "entity_type": "location", "label": "Address Line 2", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "city", "entity_type": "location", "label": "City", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "region", "entity_type": "location", "label": "Region / State", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "postal_code", "entity_type": "location", "label": "Postal Code", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "country", "entity_type": "location", "label": "Country", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "timezone", "entity_type": "location", "label": "Timezone", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "geo_coordinates", "entity_type": "location", "label": "Geo Coordinates", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 8, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "e.g. \"40.7128,-74.0060\" - plain text, no map field type exists.", "placeholder": null },
        { "key": "location_status", "entity_type": "location", "label": "Status", "field_type": "select", "options": ["Active", "Inactive"], "required": true, "show_in_list": true, "sort_order": 9, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn contact_point_fields() -> serde_json::Value {
    json!([
        { "key": "point_type", "entity_type": "contact_point", "label": "Type", "field_type": "select", "options": ["Email", "Phone", "Mobile", "Fax", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "contact_value", "entity_type": "contact_point", "label": "Value", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "purpose", "entity_type": "contact_point", "label": "Purpose", "field_type": "select", "options": ["Work", "Home", "Billing", "Other"], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "is_preferred", "entity_type": "contact_point", "label": "Preferred", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "is_verified", "entity_type": "contact_point", "label": "Verified", "field_type": "boolean", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "false", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_from", "entity_type": "contact_point", "label": "Valid From", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_to", "entity_type": "contact_point", "label": "Valid To", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn external_identifier_fields() -> serde_json::Value {
    json!([
        { "key": "external_system", "entity_type": "external_identifier", "label": "External System", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "external_key", "entity_type": "external_identifier", "label": "External Key", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "key_type", "entity_type": "external_identifier", "label": "Key Type", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Plain text, not a real relationship - see this module's own doc
        // comment on generic polymorphic relationship targets.
        { "key": "linked_object_type", "entity_type": "external_identifier", "label": "Linked Object Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - see this package's own doc comment.", "placeholder": null },
        { "key": "linked_object_id", "entity_type": "external_identifier", "label": "Linked Object ID", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_from", "entity_type": "external_identifier", "label": "Valid From", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "valid_to", "entity_type": "external_identifier", "label": "Valid To", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn consent_preference_fields() -> serde_json::Value {
    json!([
        { "key": "purpose", "entity_type": "consent_preference", "label": "Purpose", "field_type": "select", "options": ["Marketing", "Data Processing", "Communications", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "channel", "entity_type": "consent_preference", "label": "Channel", "field_type": "select", "options": ["Email", "Phone", "SMS", "Mail", "Other"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Granted consent requires a captured date" business rule below.
        { "key": "consent_state", "entity_type": "consent_preference", "label": "Consent State", "field_type": "select", "options": ["Granted", "Denied", "Withdrawn"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "captured_source", "entity_type": "consent_preference", "label": "Captured Source", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Granted by the "Granted consent requires a captured date" business rule below.
        { "key": "captured_date", "entity_type": "consent_preference", "label": "Captured Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "expiry_date", "entity_type": "consent_preference", "label": "Expiry Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn enterprise_asset_fields() -> serde_json::Value {
    json!([
        { "key": "asset_type", "entity_type": "enterprise_asset", "label": "Asset Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "asset_status", "entity_type": "enterprise_asset", "label": "Status", "field_type": "select", "options": ["Active", "Out of Service", "Retired", "Disposed"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Active", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "install_date", "entity_type": "enterprise_asset", "label": "Install Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "serial_reference", "entity_type": "enterprise_asset", "label": "Serial / Reference", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": true, "help_text": null, "placeholder": null },
        { "key": "asset_value", "entity_type": "enterprise_asset", "label": "Value", "field_type": "number", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": "0", "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn agreement_fields() -> serde_json::Value {
    json!([
        { "key": "agreement_type", "entity_type": "agreement", "label": "Agreement Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "effective_date", "entity_type": "agreement", "label": "Effective Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Checked against effective_date by the "Agreement date order" business rule below.
        { "key": "expiration_date", "entity_type": "agreement", "label": "Expiration Date", "field_type": "date", "options": [], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Agreement expired" workflow below.
        { "key": "agreement_status", "entity_type": "agreement", "label": "Status", "field_type": "select", "options": ["Draft", "Active", "Expired", "Terminated"], "required": true, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn service_case_fields() -> serde_json::Value {
    json!([
        { "key": "case_type", "entity_type": "service_case", "label": "Case Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "priority", "entity_type": "service_case", "label": "Priority", "field_type": "select", "options": ["Low", "Medium", "High", "Urgent"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Medium", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "case_status", "entity_type": "service_case", "label": "Status", "field_type": "select", "options": ["New", "In Progress", "Resolved", "Closed"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "New", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "sla_due_date", "entity_type": "service_case", "label": "SLA Due Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn interaction_fields() -> serde_json::Value {
    json!([
        { "key": "channel", "entity_type": "interaction", "label": "Channel", "field_type": "select", "options": ["Phone", "Email", "Chat", "In Person", "Other"], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "direction", "entity_type": "interaction", "label": "Direction", "field_type": "select", "options": ["Inbound", "Outbound"], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "subject", "entity_type": "interaction", "label": "Subject", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "interaction_date", "entity_type": "interaction", "label": "Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "summary", "entity_type": "interaction", "label": "Summary", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Plain text, not a real relationship - see this module's own doc
        // comment on generic polymorphic relationship targets.
        { "key": "linked_object_type", "entity_type": "interaction", "label": "Linked Object Type", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - see this package's own doc comment.", "placeholder": null },
        { "key": "linked_object_id", "entity_type": "interaction", "label": "Linked Object ID", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn document_record_fields() -> serde_json::Value {
    json!([
        { "key": "document_type", "entity_type": "document_record", "label": "Document Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "document_version", "entity_type": "document_record", "label": "Version", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Drives the "Superseded or expired document requires an expiry date" business rule below.
        { "key": "document_status", "entity_type": "document_record", "label": "Status", "field_type": "select", "options": ["Draft", "Final", "Superseded", "Expired"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Draft", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "confidentiality", "entity_type": "document_record", "label": "Confidentiality", "field_type": "select", "options": ["Public", "Internal", "Confidential", "Restricted"], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": "Internal", "is_unique": false, "help_text": "Descriptive only - nothing in this engine enforces field/record-level access by this value yet.", "placeholder": null },
        { "key": "effective_date", "entity_type": "document_record", "label": "Effective Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Superseded/Expired by the business rule below.
        { "key": "expiry_date", "entity_type": "document_record", "label": "Expiry Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Plain text, not a real relationship - see this module's own doc
        // comment on generic polymorphic relationship targets.
        { "key": "linked_object_type", "entity_type": "document_record", "label": "Linked Object Type", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - see this package's own doc comment.", "placeholder": null },
        { "key": "linked_object_id", "entity_type": "document_record", "label": "Linked Object ID", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn financial_transaction_fields() -> serde_json::Value {
    json!([
        { "key": "transaction_type", "entity_type": "financial_transaction", "label": "Transaction Type", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "amount", "entity_type": "financial_transaction", "label": "Amount", "field_type": "number", "options": [], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": true, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "currency", "entity_type": "financial_transaction", "label": "Currency", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": "USD", "is_unique": false, "help_text": null, "placeholder": null },
        // Required-when-Posted by the "Posted transaction requires a date" business rule below.
        { "key": "transaction_date", "entity_type": "financial_transaction", "label": "Transaction Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "transaction_status", "entity_type": "financial_transaction", "label": "Status", "field_type": "select", "options": ["Pending", "Posted", "Reversed"], "required": true, "show_in_list": true, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Pending", "is_unique": false, "help_text": "Descriptive only - nothing in this engine enforces immutability once Posted; a correction is an admin discipline, not a blocked edit.", "placeholder": null },
        // Plain text, not a real relationship - see this module's own doc
        // comment on generic polymorphic relationship targets.
        { "key": "linked_object_type", "entity_type": "financial_transaction", "label": "Linked Object Type", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - see this package's own doc comment.", "placeholder": null },
        { "key": "linked_object_id", "entity_type": "financial_transaction", "label": "Linked Object ID", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

fn data_quality_issue_fields() -> serde_json::Value {
    json!([
        { "key": "issue_rule", "entity_type": "data_quality_issue", "label": "Rule", "field_type": "text", "options": [], "required": true, "show_in_list": true, "sort_order": 0, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "severity", "entity_type": "data_quality_issue", "label": "Severity", "field_type": "select", "options": ["Low", "Medium", "High", "Critical"], "required": true, "show_in_list": true, "sort_order": 1, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Medium", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "issue_status", "entity_type": "data_quality_issue", "label": "Status", "field_type": "select", "options": ["Open", "In Review", "Resolved", "Ignored"], "required": true, "show_in_list": true, "sort_order": 2, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": true, "default_value": "Open", "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "detected_date", "entity_type": "data_quality_issue", "label": "Detected Date", "field_type": "date", "options": [], "required": false, "show_in_list": true, "sort_order": 3, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "resolved_date", "entity_type": "data_quality_issue", "label": "Resolved Date", "field_type": "date", "options": [], "required": false, "show_in_list": false, "sort_order": 4, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        { "key": "remediation_notes", "entity_type": "data_quality_issue", "label": "Remediation Notes", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 5, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": true, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null },
        // Plain text, not a real relationship - see this module's own doc
        // comment on generic polymorphic relationship targets. Also: this
        // package ships the storage object only, not the automatic
        // detection engine the source spec envisions - see this package's
        // own doc comment.
        { "key": "linked_object_type", "entity_type": "data_quality_issue", "label": "Linked Object Type", "field_type": "text", "options": [], "required": false, "show_in_list": true, "sort_order": 6, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": true, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": "Not a real link - see this package's own doc comment.", "placeholder": null },
        { "key": "linked_object_id", "entity_type": "data_quality_issue", "label": "Linked Object ID", "field_type": "text", "options": [], "required": false, "show_in_list": false, "sort_order": 7, "min_value": null, "max_value": null, "max_length": null, "regex_pattern": null, "is_searchable": false, "is_filterable": false, "is_reportable": false, "default_value": null, "is_unique": false, "help_text": null, "placeholder": null }
    ])
}

/// Indices below are load-bearing: `screen_layouts[0].draft`'s `related`
/// references these by position (see this module's own doc comment /
/// `IndustryPackageManifest::workflows`' doc comment). Note indices 3 and
/// 4 share the identical `(source_entity_type, target_entity_type)` pair
/// (`party_relationship` -> `party`, twice) - `relationship_service::slugify`
/// auto-suffixes the second one's key rather than rejecting the
/// collision, confirmed by reading that function directly.
fn lanesra_industry_foundation_relationships() -> serde_json::Value {
    json!([
        /* 0 */ { "source_entity_type": "party", "target_entity_type": "Contact", "relationship_type": "many_to_one", "forward_label": "Represents Contact", "reverse_label": "Parties", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 1 */ { "source_entity_type": "party", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Represents Organization", "reverse_label": "Parties", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 2 */ { "source_entity_type": "party_role", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Roles", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 3 */ { "source_entity_type": "party_relationship", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "From Party", "reverse_label": "Relationships From", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 4 */ { "source_entity_type": "party_relationship", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "To Party", "reverse_label": "Relationships To", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 1 },
        /* 5 */ { "source_entity_type": "organization_unit", "target_entity_type": "Company", "relationship_type": "many_to_one", "forward_label": "Organization", "reverse_label": "Units", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 6 */ { "source_entity_type": "location", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Locations", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 7 */ { "source_entity_type": "location", "target_entity_type": "organization_unit", "relationship_type": "many_to_one", "forward_label": "Organization Unit", "reverse_label": "Locations", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 8 */ { "source_entity_type": "contact_point", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Contact Points", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 9 */ { "source_entity_type": "consent_preference", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Consents", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 10 */ { "source_entity_type": "enterprise_asset", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Owner", "reverse_label": "Assets", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 11 */ { "source_entity_type": "enterprise_asset", "target_entity_type": "location", "relationship_type": "many_to_one", "forward_label": "Location", "reverse_label": "Assets", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 12 */ { "source_entity_type": "agreement", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Agreements", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 13 */ { "source_entity_type": "agreement", "target_entity_type": "Contract", "relationship_type": "many_to_one", "forward_label": "Related Contract", "reverse_label": "Agreements", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 1 },
        /* 14 */ { "source_entity_type": "service_case", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Cases", "is_required": true, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 },
        /* 15 */ { "source_entity_type": "interaction", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Interactions", "is_required": true, "show_related_list": true, "delete_behavior": "archive", "sort_order": 0 },
        /* 16 */ { "source_entity_type": "financial_transaction", "target_entity_type": "party", "relationship_type": "many_to_one", "forward_label": "Party", "reverse_label": "Financial Transactions", "is_required": false, "show_related_list": true, "delete_behavior": "restrict", "sort_order": 0 }
    ])
}

fn lanesra_industry_foundation_business_rules() -> serde_json::Value {
    json!([
        {
            "entity_type": "party_role",
            "name": "Role date order",
            "description": "A role's end date, once set, must be after its start date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "valid_to", "operator": "on_or_before", "value": "", "compare_field_source": "custom", "compare_field_key": "valid_from" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Valid To must be after Valid From." }
            ]
        },
        {
            "entity_type": "agreement",
            "name": "Agreement date order",
            "description": "An agreement's expiration date must be after its effective date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "expiration_date", "operator": "on_or_before", "value": "", "compare_field_source": "custom", "compare_field_key": "effective_date" }
            ],
            "actions": [
                { "action_type": "block_save", "target_field_key": null, "target_field_source": "custom", "action_value": null, "message": "Expiration date must be after the effective date." }
            ]
        },
        {
            "entity_type": "consent_preference",
            "name": "Granted consent requires a captured date",
            "description": "Granting consent must record when it was captured.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "consent_state", "operator": "equals", "value": "Granted" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "captured_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "document_record",
            "name": "Superseded or expired document requires an expiry date",
            "description": "A document marked Superseded or Expired must record when.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "document_status", "operator": "in_list", "value": "Superseded|Expired" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "expiry_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        },
        {
            "entity_type": "financial_transaction",
            "name": "Posted transaction requires a date",
            "description": "A posted financial transaction must record its transaction date.",
            "match_type": "all",
            "priority": 0,
            "effective_start_date": null,
            "effective_end_date": null,
            "conditions": [
                { "field_source": "custom", "field_key": "transaction_status", "operator": "equals", "value": "Posted" }
            ],
            "actions": [
                { "action_type": "require", "target_field_key": "transaction_date", "target_field_source": "custom", "action_value": null, "message": null }
            ]
        }
    ])
}

/// None of these reach for `update_related_record` - Foundation objects
/// relate to Party or to each other, not to a sibling record with an
/// obvious single field to overwrite the way, say, Property Management's
/// lease/unit pair does. Every workflow here creates a task instead, the
/// same fallback most packages in this file already use for the same
/// reason.
fn lanesra_industry_foundation_workflows() -> serde_json::Value {
    json!([
        {
            "entity_type": "party_role",
            "name": "Role ended",
            "description": "Ending a party role creates a task to review whatever depended on it.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "role_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "role_status", "operator": "equals", "value": "Ended" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Review dependent records for this ended role\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "agreement",
            "name": "Agreement expired",
            "description": "An expired agreement creates a task to review renewal or termination.",
            "trigger_type": "field_changed",
            "trigger_status": null,
            "trigger_field_key": "agreement_status",
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [
                { "field_source": "custom", "field_key": "agreement_status", "operator": "equals", "value": "Expired" }
            ],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Review agreement for renewal or termination\",\"description\":null,\"due_in_days\":3,\"assignee_user_id\":null}" }
            ]
        },
        {
            "entity_type": "data_quality_issue",
            "name": "New data quality issue",
            "description": "A newly created data quality issue gets a triage task.",
            "trigger_type": "record_created",
            "trigger_status": null,
            "trigger_field_key": null,
            "trigger_field_source": "custom",
            "trigger_offset_days": 0,
            "match_type": "all",
            "priority": 0,
            "conditions": [],
            "actions": [
                { "action_type": "create_task", "params_json": "{\"title\":\"Triage new data quality issue\",\"description\":null,\"due_in_days\":2,\"assignee_user_id\":null}" }
            ]
        }
    ])
}
