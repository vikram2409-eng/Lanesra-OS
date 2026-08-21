//! Raw CRUD for `integration_connectors` / `integration_connector_actions`
//! (migration 0032) - see `services::connector_service` for OpenAPI
//! parsing/validation. Mirrors `app_definition_repo`'s own "parent row and
//! child rows fetched/assembled separately" convention rather than
//! embedding a join: the `Connector` model's `actions` field is filled in
//! by the service layer from `list_actions`, not by this repo directly.

use rusqlite::Connection;

use crate::domain::ids::now_iso;
use crate::models::integration::{Connector, ConnectorAction, ConnectorActionParam};

/// The bare connector row, without its actions - callers that need the
/// full `Connector` (with `actions` populated) go through
/// `connector_service::get`/`list_for_workspace` instead.
fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Connector> {
    Ok(Connector {
        id: row.get("id")?,
        workspace_id: row.get("workspace_id")?,
        name: row.get("name")?,
        description: row.get("description")?,
        connection_type: row.get("connection_type")?,
        spec_source: row.get("spec_source")?,
        publisher_id: row.get("publisher_id")?,
        actions: Vec::new(),
        created_at: row.get("created_at")?,
        created_by: row.get("created_by")?,
        updated_at: row.get("updated_at")?,
        updated_by: row.get("updated_by")?,
    })
}

fn map_action_row(row: &rusqlite::Row) -> rusqlite::Result<ConnectorAction> {
    let params_json: String = row.get("params_json")?;
    let params: Vec<ConnectorActionParam> = serde_json::from_str(&params_json).unwrap_or_default();
    Ok(ConnectorAction {
        id: row.get("id")?,
        connector_id: row.get("connector_id")?,
        action_key: row.get("action_key")?,
        display_name: row.get("display_name")?,
        http_method: row.get("http_method")?,
        path_template: row.get("path_template")?,
        params,
        request_schema_json: row.get("request_schema_json")?,
        response_schema_json: row.get("response_schema_json")?,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    conn: &Connection,
    id: &str,
    workspace_id: &str,
    name: &str,
    description: Option<&str>,
    connection_type: &str,
    spec_source: &str,
    raw_spec: Option<&str>,
    publisher_id: Option<&str>,
    actor_user_id: Option<&str>,
) -> rusqlite::Result<Connector> {
    let now = now_iso();
    conn.execute(
        "INSERT INTO integration_connectors
            (id, workspace_id, name, description, connection_type, spec_source, raw_spec, publisher_id, created_at, created_by, updated_at, updated_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?9, ?10)",
        rusqlite::params![id, workspace_id, name, description, connection_type, spec_source, raw_spec, publisher_id, now, actor_user_id],
    )?;
    get(conn, id).map(|c| c.expect("just inserted"))
}

pub fn get(conn: &Connection, id: &str) -> rusqlite::Result<Option<Connector>> {
    conn.query_row("SELECT * FROM integration_connectors WHERE id = ?1", [id], map_row)
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn list_for_workspace(conn: &Connection, workspace_id: &str) -> rusqlite::Result<Vec<Connector>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_connectors WHERE workspace_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map([workspace_id], map_row)?;
    rows.collect()
}

pub fn delete(conn: &Connection, id: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM integration_connectors WHERE id = ?1", [id])?;
    Ok(())
}

pub fn insert_action(
    conn: &Connection,
    id: &str,
    connector_id: &str,
    action_key: &str,
    display_name: &str,
    http_method: &str,
    path_template: &str,
    params_json: &str,
    request_schema_json: Option<&str>,
    response_schema_json: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO integration_connector_actions
            (id, connector_id, action_key, display_name, http_method, path_template, params_json, request_schema_json, response_schema_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        rusqlite::params![id, connector_id, action_key, display_name, http_method, path_template, params_json, request_schema_json, response_schema_json, now_iso()],
    )?;
    Ok(())
}

pub fn list_actions(conn: &Connection, connector_id: &str) -> rusqlite::Result<Vec<ConnectorAction>> {
    let mut stmt = conn.prepare("SELECT * FROM integration_connector_actions WHERE connector_id = ?1 ORDER BY action_key")?;
    let rows = stmt.query_map([connector_id], map_action_row)?;
    rows.collect()
}

pub fn get_action(conn: &Connection, connector_id: &str, action_key: &str) -> rusqlite::Result<Option<ConnectorAction>> {
    conn.query_row(
        "SELECT * FROM integration_connector_actions WHERE connector_id = ?1 AND action_key = ?2",
        rusqlite::params![connector_id, action_key],
        map_action_row,
    )
    .map(Some)
    .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}
