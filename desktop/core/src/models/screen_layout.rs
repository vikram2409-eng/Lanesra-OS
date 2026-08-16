use serde::{Deserialize, Serialize};

/// Screen/App Builder Phase 1: an object (a built-in entity_type name or
/// a custom object's key) can have several named layouts, each with its
/// own tabs of admin-drag-ordered field sections - see the migration's
/// header comment for why this layer stores fields as opaque key strings
/// rather than coupling to any specific field registry.
#[derive(Debug, Clone, Serialize)]
pub struct ScreenLayout {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub is_default: bool,
    /// Roles this layout is shown to - see
    /// `screen_layout_service::resolve_for_roles`. Ignored on the default
    /// layout, which is the fallback for any role no other layout claims.
    pub roles: Vec<String>,
    pub draft: LayoutTabs,
    /// `None` until first published - the live create/edit (and later
    /// detail) screens only ever render this, never the draft.
    pub published: Option<LayoutTabs>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LayoutTabs {
    pub tabs: Vec<LayoutTab>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutTab {
    pub id: String,
    pub title: String,
    pub sections: Vec<LayoutSection>,
    /// Screen/App Builder Phase 3: relationship-definition keys (see
    /// `RelationshipDefinition::key` - this layer stores them the same
    /// opaque-string way it stores field keys) whose related-records list
    /// renders on this tab. A key that isn't on any tab's `related` still
    /// isn't hidden - the frontend falls back to showing it in an
    /// always-visible spot outside the tab strip, the same "never
    /// silently drop something the layout doesn't know about" rule
    /// `LayoutFormFields` already applies to unplaced fields. `#[serde(default)]`
    /// so a layout saved before this phase (no `related` key at all)
    /// still loads.
    #[serde(default)]
    pub related: Vec<String>,
}

/// Screen/App Builder Phase 2: a section lays its fields out in a CSS
/// grid of `columns` columns (1-3); each field spans either one column or
/// the section's full width. `columns` defaults to 2 (the fixed width
/// every Phase 1 form already used) when absent from stored JSON, and
/// `fields` accepts either the Phase 1 shape (`["key", ...]`, each
/// implicitly one column wide) or the Phase 2 shape
/// (`[{"key":"...", "full_width":false}, ...]`) so a layout saved before
/// this phase shipped still loads - see `deserialize_fields` below.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSection {
    pub id: String,
    pub title: String,
    #[serde(default = "default_columns")]
    pub columns: u8,
    #[serde(deserialize_with = "deserialize_fields")]
    pub fields: Vec<SectionField>,
}

fn default_columns() -> u8 {
    2
}

/// One field placed in a section: its key, and whether it spans the
/// section's full width (like a name or notes field usually does) rather
/// than a single column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionField {
    pub key: String,
    #[serde(default)]
    pub full_width: bool,
}

impl SectionField {
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), full_width: false }
    }
}

impl From<&str> for SectionField {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

/// Accepts a plain field-key string (the Phase 1 wire format, defaulting
/// to a single-column field) alongside the Phase 2 `{key, full_width}`
/// object shape, so existing draft/published JSON keeps loading as-is.
fn deserialize_fields<'de, D>(deserializer: D) -> Result<Vec<SectionField>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawField {
        Key(String),
        Full(SectionField),
    }
    let raw = Vec::<RawField>::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .map(|f| match f {
            RawField::Key(key) => SectionField::new(key),
            RawField::Full(field) => field,
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenLayoutInput {
    pub entity_type: String,
    pub name: String,
    /// Field keys to pre-populate into the new layout's first tab/section.
    /// This layer stores fields as opaque key strings (see the
    /// migration's own comment) and has no registry of what fields an
    /// entity type actually has, built-in or custom - the frontend does,
    /// so it passes the same "every current field" list the online
    /// demo's `freshLayout` pre-populates a new layout with, rather than
    /// starting the admin from a blank slate. May be empty.
    pub initial_fields: Vec<String>,
}

/// Covers rename, role reassignment, and any draft edit (tab/section/
/// field changes) in one save - the builder UI edits all of these
/// together and there's no independent reason to split them.
#[derive(Debug, Clone, Deserialize)]
pub struct ScreenLayoutUpdate {
    pub name: String,
    pub roles: Vec<String>,
    pub draft: LayoutTabs,
}
