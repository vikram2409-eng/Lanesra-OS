//! A thin, synchronous wrapper around a Team Workspace server's
//! `/api/v1` REST API (`server/src/api_v1.rs`) - the same credential
//! (`Authorization: Bearer {client_id}.{secret}`, issued from Admin ->
//! Integration Hub -> API Access) that surface already authenticates.
//! No business logic lives here: every call is a plain HTTP request,
//! and every check (permission, validation, scope) already happens
//! server-side through `api_object_service`, exactly as it does for any
//! other REST caller.

use reqwest::blocking::{Client, Response};
use reqwest::StatusCode;
use serde_json::Value;

pub struct ApiClient {
    base_url: String,
    api_key: String,
    http: Client,
}

/// A REST response as-is: the caller decides how to present a non-2xx
/// body (`{"ok": false, "error": ...}`, the same shape every `api_v1.rs`
/// route already returns) rather than this module deciding for it.
pub struct ApiResponse {
    pub status: StatusCode,
    pub body: Value,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self { base_url: base_url.into().trim_end_matches('/').to_string(), api_key: api_key.into(), http: Client::new() }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn to_response(&self, resp: Response) -> Result<ApiResponse, String> {
        let status = resp.status();
        let body: Value = resp.json().map_err(|e| format!("could not parse response body as JSON: {e}"))?;
        Ok(ApiResponse { status, body })
    }

    pub fn get(&self, path: &str) -> Result<ApiResponse, String> {
        let resp = self.http.get(self.url(path)).bearer_auth(&self.api_key).send().map_err(|e| e.to_string())?;
        self.to_response(resp)
    }

    pub fn get_with_query(&self, path: &str, query: &[(&str, String)]) -> Result<ApiResponse, String> {
        let resp = self.http.get(self.url(path)).bearer_auth(&self.api_key).query(query).send().map_err(|e| e.to_string())?;
        self.to_response(resp)
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<ApiResponse, String> {
        let resp = self.http.post(self.url(path)).bearer_auth(&self.api_key).json(body).send().map_err(|e| e.to_string())?;
        self.to_response(resp)
    }

    pub fn patch(&self, path: &str, body: &Value) -> Result<ApiResponse, String> {
        let resp = self.http.patch(self.url(path)).bearer_auth(&self.api_key).json(body).send().map_err(|e| e.to_string())?;
        self.to_response(resp)
    }

    pub fn delete(&self, path: &str) -> Result<ApiResponse, String> {
        let resp = self.http.delete(self.url(path)).bearer_auth(&self.api_key).send().map_err(|e| e.to_string())?;
        self.to_response(resp)
    }
}
