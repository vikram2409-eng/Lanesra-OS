//! Screen Builder 2.0 (issue #195, 5b): "admins can save a customized page
//! as an Organization Template," offered alongside the 4 built-in
//! templates (pure client-side data - see `pageTemplates.ts`) in the same
//! "start from a template" picker. A template row is an immutable
//! snapshot - there's no update/publish here, only create/list/delete.

use rusqlite::Connection;

use crate::domain::{AppError, AppResult};
use crate::models::page_template::{PageTemplate, PageTemplateInput};
use crate::repositories::page_template_repo;

fn require_admin(conn: &Connection, actor_user_id: Option<&str>) -> AppResult<()> {
    super::user_service::require_admin(conn, actor_user_id)
}

fn hydrate(row: (PageTemplate, String)) -> AppResult<PageTemplate> {
    let (mut template, definition_json) = row;
    template.definition = serde_json::from_str(&definition_json).unwrap_or_default();
    Ok(template)
}

pub fn list_templates(conn: &Connection, workspace_id: &str, entity_type: &str) -> AppResult<Vec<PageTemplate>> {
    page_template_repo::list(conn, workspace_id, entity_type)?.into_iter().map(hydrate).collect()
}

/// Snapshots `page_id`'s *current draft* (not its published tree, if any -
/// a template captures exactly what the admin is looking at in the
/// builder right now) into a brand-new, independent template row.
pub fn create_template_from_page(
    conn: &Connection,
    workspace_id: &str,
    page_id: &str,
    input: &PageTemplateInput,
    actor_user_id: Option<&str>,
) -> AppResult<PageTemplate> {
    require_admin(conn, actor_user_id)?;
    if input.name.trim().is_empty() {
        return Err(AppError::Validation("Template name is required".into()));
    }
    let page = super::page_layout_service::get_layout(conn, page_id)?;
    if page.workspace_id != workspace_id {
        return Err(AppError::NotFound("Page layout".into()));
    }
    let id = page_template_repo::new_id();
    let definition_json = serde_json::to_string(&page.draft).expect("PageDefinition always serializes");
    page_template_repo::create(
        conn,
        &id,
        workspace_id,
        &page.entity_type,
        input.name.trim(),
        input.description.as_deref(),
        &definition_json,
        actor_user_id,
    )?;
    hydrate(page_template_repo::get(conn, &id)?.ok_or_else(|| AppError::NotFound("Page template".into()))?)
}

pub fn delete_template(conn: &Connection, id: &str, workspace_id: &str, actor_user_id: Option<&str>) -> AppResult<()> {
    require_admin(conn, actor_user_id)?;
    let (template, _) = page_template_repo::get(conn, id)?.ok_or_else(|| AppError::NotFound("Page template".into()))?;
    if template.workspace_id != workspace_id {
        return Err(AppError::NotFound("Page template".into()));
    }
    page_template_repo::delete(conn, id)?;
    Ok(())
}
