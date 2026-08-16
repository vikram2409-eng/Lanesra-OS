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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayoutSection {
    pub id: String,
    pub title: String,
    pub fields: Vec<String>,
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
