//! Industry Data Model foundations - see migration 0027's own comment for
//! the registry-table side of this. This module is the manifest format
//! itself (dev spec "Top 10 Industry Data Models & Packaged Business
//! Apps", section 1.1 "Minimum package manifest") plus the two registry
//! read models (`InstalledApp`, `PackageArtifact`) the frontend and
//! `industry_package_service` work with.
//!
//! Deliberately NOT built yet (foundation phase only - see
//! `industry_package_service`'s own module comment for the full list):
//! update-with-diff preserving admin customizations, destructive
//! uninstall, and `is_locally_customized` drift detection. The schema and
//! manifest shape below are written to hold those without another
//! migration once they're built.

use serde::{Deserialize, Serialize};

use crate::models::business_rule::BusinessRuleInput;
use crate::models::custom_field::{CustomFieldDefinitionInput, CustomFieldValues};
use crate::models::custom_object::CustomObjectDefinitionInput;
use crate::models::custom_record::CustomRecordInput;
use crate::models::custom_report::CustomReportInput;
use crate::models::numbering_override::NumberingOverrideInput;
use crate::models::relationship::RelationshipDefinitionInput;
use crate::models::screen_layout::LayoutTabs;
use crate::models::workflow::WorkflowDefinitionInput;

/// The manifest *schema's* version (bumped only when this struct's own
/// shape changes) - mirrors `BackupManifest::format_version`'s identical
/// reasoning, kept separate from any one package's own `version`.
pub const INDUSTRY_PACKAGE_FORMAT_VERSION: u32 = 1;

/// One custom object a package defines, with an explicit, deterministic
/// key the package author chooses. This can't be left to the admin UI's
/// auto-slugify-and-uniquify (`custom_object_service::create`'s
/// `slugify`): every other part of this same manifest that names this
/// object (a field's `entity_type`, a relationship's source/target, a
/// business rule/workflow's `entity_type`, a screen layout's
/// `entity_type`, a seed record's `object_key`) references it by this
/// exact string, so a silent "_2" suffix on a name collision would
/// silently break every one of those references instead of failing loud
/// at install time the way spec 13.3's conflict policy requires.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestObject {
    pub key: String,
    #[serde(flatten)]
    pub definition: CustomObjectDefinitionInput,
}

/// Same determinism reasoning as `ManifestObject` - a field's key is what
/// a business rule/workflow's `field_key`/`target_field_key` names it by,
/// so it must be explicit and collision-checked per entity_type, not
/// derived from `label` the way `custom_field_service::create_definition`
/// derives it for the admin UI. `entity_type` (on the flattened
/// `CustomFieldDefinitionInput`) can name a built-in type, one of this
/// same manifest's own `objects`, or an object another already-installed
/// package/admin created - that last case is exactly spec 13.3's "a
/// package may extend a core or existing object" allowance; nothing
/// beyond the field's own key needs to be unique for an extension, only
/// the (entity_type, key) pair.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestField {
    pub key: String,
    #[serde(flatten)]
    pub definition: CustomFieldDefinitionInput,
}

/// A screen layout to seed (and optionally publish) for one entity_type.
/// `related` names relationships by their position in
/// `IndustryPackageManifest::relationships` (0-based) rather than by key,
/// since a `RelationshipDefinition`'s key auto-derives from its
/// source/target labels at install time and isn't something the package
/// author controls the way `ManifestObject`/`ManifestField` keys are.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestScreenLayout {
    pub entity_type: String,
    pub name: String,
    pub draft: LayoutTabs,
    #[serde(default)]
    pub publish: bool,
}

/// The one dashboard a package can ship. A `"chart"` widget's config
/// carries `report_ref` (an index into `IndustryPackageManifest::reports`)
/// instead of a real `report_id`, resolved by the installer once that
/// report actually exists - the same index-reference reasoning
/// `ManifestScreenLayout::related` uses for relationships, since neither
/// has a real id yet at manifest-authoring time.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestDashboard {
    pub name: String,
    pub widgets: Vec<ManifestDashboardWidget>,
    #[serde(default)]
    pub publish: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestDashboardWidget {
    pub kind: String,
    pub config: serde_json::Value,
}

/// spec 1.1's "permissions: Recommended role templates, always reviewed
/// by administrator before activation" - install stores these as
/// read-only guidance on the `InstalledApp` (see
/// `installed_apps.recommended_permissions_json`); it never creates a
/// real `AppPermission` grant from one automatically. `role` is one of
/// `user_repo::ROLES`; `level` is `"viewer"` or `"editor"` (see
/// `models::app_definition::APP_PERMISSION_LEVELS`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedPermission {
    pub role: String,
    pub level: String,
}

/// spec 1.1's "navigation: App landing page, menu order and default
/// views" - a package's own App Builder grouping (see migration 0025).
/// `object_keys` (built-in type names and/or this manifest's own
/// `objects[].key`s) becomes the created `AppDefinition.object_keys`,
/// whose order *is* the app's menu order (App Builder has no separate
/// ordering concept to duplicate here). `use_package_dashboard` wires
/// `IndustryPackageManifest::dashboard`, once created, up as this app's
/// own Dashboard section. "Default views" (spec 1.1) has no home in the
/// platform yet - there's no saved-view concept built at all - so it's
/// deliberately not modeled here; see `industry_package_service`'s own
/// module comment.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestApp {
    pub name: String,
    pub icon: String,
    pub description: Option<String>,
    pub object_keys: Vec<String>,
    #[serde(default)]
    pub use_package_dashboard: bool,
    #[serde(default)]
    pub publish: bool,
    #[serde(default)]
    pub recommended_permissions: Vec<RecommendedPermission>,
}

/// spec 1.1's "seed_data: Reference data only; production customer
/// sample data off by default" - a starter record for one of this
/// package's own objects. `object_key` must name an entry in
/// `IndustryPackageManifest::objects` (seed data for a built-in or
/// someone else's object is out of scope - this manifest doesn't own
/// enough of that record's shape to seed it safely).
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestSampleRecord {
    pub object_key: String,
    pub record: CustomRecordInput,
    #[serde(default)]
    pub field_values: CustomFieldValues,
}

/// A required or optional dependency on another package - spec 13.2's
/// `app_dependencies` shape. `version_constraint` supports only `"*"`
/// (any version) or `">=X.Y.Z"` for this foundation phase (see
/// `industry_package_service::version_satisfies` for the exact, minimal
/// comparison rule) - a full semver range grammar is future scope.
#[derive(Debug, Clone, Deserialize)]
pub struct ManifestDependency {
    pub package_id: String,
    pub version_constraint: String,
    #[serde(default = "default_true")]
    pub is_required: bool,
}

fn default_true() -> bool {
    true
}

/// The full installable package - dev spec section 1.1's manifest,
/// applied in one transaction by `industry_package_service::install`.
/// `package_id` is the package's own stable identity (e.g.
/// `"lanesra.field_service"`, spec's own example) - constant across every
/// version of this package, unlike `IndustryPackageManifest::name` which
/// can be relabeled. `format_version` is this *struct's* schema version
/// (see `INDUSTRY_PACKAGE_FORMAT_VERSION`), not the package's own
/// `version`.
#[derive(Debug, Clone, Deserialize)]
pub struct IndustryPackageManifest {
    pub format_version: u32,
    pub package_id: String,
    pub name: String,
    pub industry: String,
    pub version: String,
    pub min_lanesra_version: String,
    #[serde(default)]
    pub dependencies: Vec<ManifestDependency>,
    #[serde(default)]
    pub objects: Vec<ManifestObject>,
    #[serde(default)]
    pub fields: Vec<ManifestField>,
    #[serde(default)]
    pub relationships: Vec<RelationshipDefinitionInput>,
    #[serde(default)]
    pub business_rules: Vec<BusinessRuleInput>,
    #[serde(default)]
    pub workflows: Vec<WorkflowDefinitionInput>,
    #[serde(default)]
    pub screen_layouts: Vec<ManifestScreenLayout>,
    pub dashboard: Option<ManifestDashboard>,
    #[serde(default)]
    pub reports: Vec<CustomReportInput>,
    #[serde(default)]
    pub numbering_overrides: Vec<NumberingOverrideInput>,
    pub app: Option<ManifestApp>,
    #[serde(default)]
    pub seed_data: Vec<ManifestSampleRecord>,
}

// --- Registry read models -------------------------------------------------

/// A package imported into this workspace's local catalog (spec 13.2's
/// `app_packages`) - validated and available to install, not yet
/// necessarily installed.
#[derive(Debug, Clone, Serialize)]
pub struct AppPackage {
    pub id: String,
    pub workspace_id: String,
    pub package_id: String,
    pub name: String,
    pub industry: String,
    pub version: String,
    pub min_lanesra_version: String,
    pub manifest_json: String,
    pub checksum: String,
    pub source: String,
    pub imported_at: String,
    pub imported_by: Option<String>,
}

pub const INSTALLED_APP_STATUSES: &[&str] = &["active", "deactivated"];

/// One package actually installed into this workspace - spec 13.2's
/// `installed_apps`.
#[derive(Debug, Clone, Serialize)]
pub struct InstalledApp {
    pub id: String,
    pub workspace_id: String,
    pub package_id: String,
    pub name: String,
    pub icon: String,
    pub industry: String,
    pub description: Option<String>,
    pub installed_version: String,
    pub status: String,
    pub app_definition_id: Option<String>,
    pub recommended_permissions: Vec<RecommendedPermission>,
    pub installed_at: String,
    pub installed_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    pub deactivated_at: Option<String>,
    pub deactivated_by: Option<String>,
}

/// One record an install created - spec 13.2's `package_artifacts`. See
/// migration 0027's own comment on `artifact_type`'s fixed vocabulary and
/// what `metadata_id` holds for the one artifact type (numbering
/// overrides) with no id of its own.
#[derive(Debug, Clone, Serialize)]
pub struct PackageArtifact {
    pub id: String,
    pub installed_app_id: String,
    pub artifact_type: String,
    pub metadata_id: String,
    pub origin_version: String,
    pub is_locally_customized: bool,
    pub created_at: String,
}

pub const APP_INSTALL_RUN_ACTIONS: &[&str] = &["install", "update", "deactivate", "reactivate"];
pub const APP_INSTALL_RUN_STATUSES: &[&str] = &["running", "succeeded", "failed"];

/// One install/update/deactivate/reactivate attempt, success or failure -
/// spec 13.2's `app_install_runs`.
#[derive(Debug, Clone, Serialize)]
pub struct AppInstallRun {
    pub id: String,
    pub workspace_id: String,
    pub package_id: String,
    pub package_version: String,
    pub action: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub backup_snapshot_path: Option<String>,
    pub error_message: Option<String>,
    pub actor_user_id: Option<String>,
}

/// A package with everything installed for it, for the Admin -> App
/// Catalog detail view - avoids the frontend making a second round trip
/// just to show what an install actually created.
#[derive(Debug, Clone, Serialize)]
pub struct InstalledAppDetail {
    pub app: InstalledApp,
    pub artifacts: Vec<PackageArtifact>,
}

/// The request body for `industry_package_service::import_package` -
/// just the raw manifest JSON text, the same "paste or upload a file"
/// shape the Admin -> App Catalog screen collects (spec: "Admin -> Apps &
/// Industry Models -> App Catalog -> Review -> Validate -> Install").
#[derive(Debug, Clone, Deserialize)]
pub struct ImportPackageInput {
    pub manifest_json: String,
}
