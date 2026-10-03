use serde::{Deserialize, Serialize};

/// Screen Builder 2.0 (issue #195, 5a): a responsive page composer, stored
/// alongside (not instead of) the Phase 1-3 `screen_layouts` system - see
/// the migration's header comment for why this is a second table rather
/// than an evolution of `ScreenLayout`'s tabs/sections/columns tree.
///
/// Same draft/published/default/roles governance shape as `ScreenLayout`
/// on purpose: the lifecycle (draft auto-saves, an explicit Publish makes
/// it live, Unpublish/Revert/make-default/delete all behave identically)
/// is proven and this issue doesn't ask for a different one - only the
/// tree *inside* draft/published is new.
#[derive(Debug, Clone, Serialize)]
pub struct PageLayout {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub is_default: bool,
    /// Same semantics as `ScreenLayout::roles` - see
    /// `page_layout_service::resolve_effective_page`.
    pub roles: Vec<String>,
    pub draft: PageDefinition,
    /// `None` until first published. 5a ships the builder only; nothing
    /// reads `published` to actually render a live record's detail page
    /// yet - that wiring is 5b, same as the issue's own split.
    pub published: Option<PageDefinition>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

/// A page is a flat, ordered list of root-level nodes stacked vertically -
/// each one (almost always a `section` or `grid`, but not enforced here;
/// see `page_layout_service::COMPONENT_TYPES`) spans the full 12-column
/// row width by convention, with its own children arranging themselves on
/// the grid inside it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PageDefinition {
    pub root: Vec<PageNode>,
}

/// One placed component. `component_type` is validated against the
/// service layer's own allowlist (`page_layout_service::COMPONENT_TYPES`)
/// on every save - this model layer stays as agnostic to what a
/// component *is* as `ScreenLayout` already is to what a field is, so
/// adding a new component type later never needs a migration.
///
/// `config` is intentionally `serde_json::Value`, not a strict struct -
/// each component type's own shape (a Field's target key, a KPI's metric
/// choice, a Button's label/action) is heterogeneous enough that one
/// sum-typed enum would just reinvent a dynamic value with extra steps.
/// The frontend's component-library schema (`pageComponentLibrary.ts`)
/// is the single source of truth for what's actually inside `config` for
/// a given `component_type`; the service layer validates presence/shape
/// only where getting it wrong would break rendering outright (see
/// `page_layout_service::validate_node`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageNode {
    pub id: String,
    pub component_type: String,
    #[serde(default = "default_config")]
    pub config: serde_json::Value,
    /// Only meaningful for a container type (see
    /// `page_layout_service::CONTAINER_COMPONENT_TYPES`) - a leaf type's
    /// `children` is always empty, enforced by `validate_node`, not by a
    /// separate leaf/container struct split. Keeping one uniform node
    /// shape (rather than an enum of "container node" vs "leaf node")
    /// means the canvas, the inspector and this model all walk the same
    /// recursive tree shape everywhere.
    #[serde(default)]
    pub children: Vec<PageNode>,
    pub layout: NodeLayout,
}

fn default_config() -> serde_json::Value {
    serde_json::json!({})
}

/// A node's column span (1-12) per breakpoint. `tablet`/`mobile` are
/// `None` by default, not a duplicate of `desktop` - `None` means "use
/// the next breakpoint up's value, capped at 12", so a brand-new node
/// reflows sensibly without an admin having to set all three explicitly;
/// the frontend resolves the cascade (desktop -> tablet -> mobile) once
/// and shows the *effective* span in the inspector, but what's actually
/// stored is only the breakpoints an admin deliberately overrode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLayout {
    #[serde(default = "default_column_span")]
    pub column_span: u8,
    #[serde(default)]
    pub tablet_column_span: Option<u8>,
    #[serde(default)]
    pub mobile_column_span: Option<u8>,
    /// Explicit order within the parent's `children` - redundant with
    /// `Vec` position for anything built through the canvas (which always
    /// keeps both in sync), but kept as a real field rather than relying
    /// on array order alone so a future drag-reorder implementation has
    /// somewhere to express "moved" without rewriting sibling indices.
    #[serde(default)]
    pub order: i32,
}

fn default_column_span() -> u8 {
    12
}

impl Default for NodeLayout {
    fn default() -> Self {
        Self { column_span: default_column_span(), tablet_column_span: None, mobile_column_span: None, order: 0 }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageLayoutInput {
    pub entity_type: String,
    pub name: String,
}

/// Covers rename, role reassignment, and any draft tree edit in one save -
/// same rationale as `ScreenLayoutUpdate`.
#[derive(Debug, Clone, Deserialize)]
pub struct PageLayoutUpdate {
    pub name: String,
    pub roles: Vec<String>,
    pub draft: PageDefinition,
}
