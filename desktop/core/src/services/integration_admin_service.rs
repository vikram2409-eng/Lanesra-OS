//! WP-0189: integration administration on the REST API.
//!
//! `admin.integration.read` lists connections, connectors, jobs and API
//! clients as metadata only: never a secret, ciphertext, token, key hash or
//! credentials in a URL. `admin.integration.manage` pauses and reactivates
//! a connection or a job and runs a job now; it cannot create anything,
//! read a secret or change a scope. The server checks the scope before
//! calling here, so these functions do not ask for an Administrator; each
//! change is audited under the API client's id.

use rusqlite::Connection;
use serde_json::json;

use crate::domain::AppResult;
use crate::models::integration::{
    ApiClient, ApiConnectionSummary, ApiConnectorSummary, Connection as ConnectionModel,
    IntegrationJob, IntegrationJobRun,
};
use crate::repositories::{audit_repo, integration_connection_repo, integration_job_repo};
use crate::services::connection_service::CONNECTION_PAUSED;
use crate::services::{
    api_client_service, connection_service, connector_service, integration_job_service,
};

/// The API client making a change, for the audit log.
#[derive(Debug, Clone, Copy)]
pub struct ApiCaller<'a> {
    pub id: &'a str,
    pub name: &'a str,
}

impl ApiCaller<'_> {
    fn audit(
        &self,
        conn: &Connection,
        workspace_id: &str,
        event_type: &str,
        entity_type: &str,
        entity_id: &str,
        summary: &str,
    ) -> AppResult<()> {
        audit_repo::record(
            conn,
            workspace_id,
            None,
            event_type,
            Some(entity_type),
            Some(entity_id),
            &format!("{summary} (API client '{}')", self.name),
            Some(&json!({"api_client_id": self.id, "api_client_name": self.name}).to_string()),
        )?;
        Ok(())
    }
}

/// A base URL without anything that could be a credential: user info,
/// query string and fragment are dropped.
fn safe_url(url: Option<&str>) -> Option<String> {
    let url = url?.trim();
    if url.is_empty() {
        return None;
    }
    match reqwest::Url::parse(url) {
        Ok(mut parsed) => {
            let _ = parsed.set_username("");
            let _ = parsed.set_password(None);
            parsed.set_query(None);
            parsed.set_fragment(None);
            Some(parsed.to_string())
        }
        // Not a URL: show nothing rather than something unchecked.
        Err(_) => None,
    }
}

fn connection_summary(c: ConnectionModel) -> ApiConnectionSummary {
    ApiConnectionSummary {
        base_url: safe_url(c.base_url.as_deref()),
        id: c.id,
        name: c.name,
        connection_type: c.connection_type,
        auth_mode: c.auth_mode,
        has_secret: c.has_secret,
        status: c.status,
        last_test_at: c.last_test_at,
        last_test_status: c.last_test_status,
        created_at: c.created_at,
        updated_at: c.updated_at,
    }
}

pub fn list_connections(
    conn: &Connection,
    workspace_id: &str,
) -> AppResult<Vec<ApiConnectionSummary>> {
    Ok(
        integration_connection_repo::list_for_workspace(conn, workspace_id)?
            .into_iter()
            .map(connection_summary)
            .collect(),
    )
}

pub fn list_connectors(
    conn: &Connection,
    workspace_id: &str,
) -> AppResult<Vec<ApiConnectorSummary>> {
    Ok(connector_service::list_for_workspace(conn, workspace_id)?
        .into_iter()
        .map(|c| ApiConnectorSummary {
            action_count: c.actions.len(),
            id: c.id,
            name: c.name,
            description: c.description,
            connection_type: c.connection_type,
            agent_tools_enabled: c.agent_tools_enabled,
            agent_write_tools_enabled: c.agent_write_tools_enabled,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect())
}

pub fn list_jobs(conn: &Connection, workspace_id: &str) -> AppResult<Vec<IntegrationJob>> {
    integration_job_service::list_for_workspace(conn, workspace_id)
}

/// API clients carry no key or hash in their model; only `sha256` of the
/// secret is stored, in a column `ApiClient` does not map.
pub fn list_api_clients(conn: &Connection, workspace_id: &str) -> AppResult<Vec<ApiClient>> {
    api_client_service::list_for_workspace(conn, workspace_id)
}

/// Pauses (no outbound call until reactivated) or reactivates a connection.
pub fn set_connection_paused(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    paused: bool,
    caller: ApiCaller,
) -> AppResult<ApiConnectionSummary> {
    let connection = connection_service::get(conn, workspace_id, id)?;
    let status = if paused { CONNECTION_PAUSED } else { "active" };
    integration_connection_repo::set_status(conn, id, status)?;
    caller.audit(
        conn,
        workspace_id,
        if paused {
            "integration_connection.paused"
        } else {
            "integration_connection.reactivated"
        },
        "integration_connection",
        id,
        &format!("Connection '{}' is now {status}", connection.name),
    )?;
    Ok(connection_summary(connection_service::get(
        conn,
        workspace_id,
        id,
    )?))
}

/// Pauses (the scheduler skips it) or reactivates a job.
pub fn set_job_paused(
    conn: &Connection,
    workspace_id: &str,
    id: &str,
    paused: bool,
    caller: ApiCaller,
) -> AppResult<IntegrationJob> {
    let job = integration_job_service::get(conn, workspace_id, id)?;
    let status = if paused { "paused" } else { "active" };
    integration_job_repo::set_status(conn, id, status)?;
    caller.audit(
        conn,
        workspace_id,
        if paused {
            "integration_job.paused"
        } else {
            "integration_job.reactivated"
        },
        "integration_job",
        id,
        &format!("Integration job '{}' is now {status}", job.name),
    )?;
    integration_job_service::get(conn, workspace_id, id)
}

/// Runs a job now, whatever its schedule; the run is in the integration log.
pub async fn run_job_now(
    conn: &Connection,
    workspace_id: &str,
    master_key: &[u8; 32],
    id: &str,
    caller: ApiCaller<'_>,
) -> AppResult<IntegrationJobRun> {
    let job = integration_job_service::get(conn, workspace_id, id)?;
    caller.audit(
        conn,
        workspace_id,
        "integration_job.run_requested",
        "integration_job",
        id,
        &format!("Ran integration job '{}' now", job.name),
    )?;
    integration_job_service::run_now_for_api_client(conn, workspace_id, master_key, id).await
}

#[cfg(test)]
mod tests {
    use super::safe_url;

    #[test]
    fn a_base_url_loses_anything_that_could_be_a_credential() {
        // Covers: REQ-INT-005 (WP-0189 AC-1)
        let (user, pass) = ("svc", ["hunter", "two"].join("-"));
        let url = format!("https://{user}:{pass}@api.example.com/v1?token=abc#frag");
        assert_eq!(
            safe_url(Some(&url)).as_deref(),
            Some("https://api.example.com/v1")
        );
        assert_eq!(safe_url(Some("not a url")), None);
        assert_eq!(safe_url(None), None);
    }
}
