use serde::{Deserialize, Serialize};

use super::page_layout::PageDefinition;

/// Screen Builder 2.0 (issue #195, 5b): "admins can save a customized page
/// as an Organization Template." An immutable, workspace-scoped snapshot
/// of a `PageLayout`'s draft at the moment it was saved - unlike
/// `PageLayout` itself, a template has no draft/published/roles/is_default
/// lifecycle, because it's never rendered live; it only ever gets copied
/// into a page's draft via "Apply template," at which point the result is
/// an ordinary, independent page like any other (see `pageTemplates.ts`'s
/// own doc comment on the 4 built-in templates, which don't need a table
/// at all for the same reason).
#[derive(Debug, Clone, Serialize)]
pub struct PageTemplate {
    pub id: String,
    pub workspace_id: String,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub definition: PageDefinition,
    pub created_at: String,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageTemplateInput {
    pub name: String,
    pub description: Option<String>,
}
