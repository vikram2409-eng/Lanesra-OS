//! Self-hosted internet deployment (roadmap item 2): config for cookie
//! security, CORS, and response security headers when this process is
//! exposed on the open internet behind a reverse proxy rather than kept
//! LAN-only, the original PRD scope for Team Workspace mode - see the
//! README's reverse-proxy/TLS recipe. Every field's default reproduces
//! that original LAN-only behavior exactly, so an existing deployment is
//! unaffected unless someone explicitly opts in.

use axum::extract::{Request, State};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

#[derive(Debug, Clone, Default)]
pub struct SecurityConfig {
    /// Mark the session cookie `Secure` and send `Strict-Transport-Security`.
    /// Only turn this on once a reverse proxy is actually terminating TLS
    /// in front of this process - a `Secure` cookie handed to a browser
    /// that only ever sees the plain-HTTP hop between it and the proxy is
    /// simply never sent back, which silently breaks every session rather
    /// than failing loudly.
    pub trust_proxy_https: bool,
    /// Origins allowed to call this API cross-origin with credentials
    /// (the session cookie) included. Empty means no CORS layer is added
    /// at all - this server always serves its own frontend same-origin,
    /// so that default is fully functional; only set this if the frontend
    /// is served from a different origin than this API (e.g. a separate
    /// static-hosting deployment).
    pub allowed_origins: Vec<String>,
}

impl SecurityConfig {
    /// Reads `LANESRA_TRUST_PROXY_HTTPS` ("1"/"true", case-insensitive)
    /// and `LANESRA_ALLOWED_ORIGINS` (comma-separated) from the
    /// environment - see the README's reverse-proxy/TLS recipe for when
    /// to set either. Unset means the original LAN-only defaults.
    pub fn from_env() -> Self {
        let trust_proxy_https = std::env::var("LANESRA_TRUST_PROXY_HTTPS")
            .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true"))
            .unwrap_or(false);
        let allowed_origins = std::env::var("LANESRA_ALLOWED_ORIGINS")
            .map(|v| v.split(',').map(str::trim).filter(|s| !s.is_empty()).map(String::from).collect())
            .unwrap_or_default();
        SecurityConfig { trust_proxy_https, allowed_origins }
    }
}

/// A conservative, always-on header set: MIME sniffing off and no framing
/// by another site (this app has no legitimate reason to be iframed), plus
/// a trimmed-down Referer. `Strict-Transport-Security` is added only when
/// `trust_proxy_https` is set - sending it over a connection that's
/// actually plain HTTP end-to-end (the LAN-only default) would be
/// meaningless, and browsers ignore it over plain HTTP regardless, so
/// gating it just keeps the response honest about what's actually true.
pub async fn security_headers(State(config): State<SecurityConfig>, request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(HeaderName::from_static("x-content-type-options"), HeaderValue::from_static("nosniff"));
    headers.insert(HeaderName::from_static("x-frame-options"), HeaderValue::from_static("DENY"));
    headers.insert(HeaderName::from_static("referrer-policy"), HeaderValue::from_static("strict-origin-when-cross-origin"));
    if config.trust_proxy_https {
        headers.insert(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        );
    }
    response
}

/// `None` when `allowed_origins` is empty - see `SecurityConfig::allowed_origins`'s
/// own doc comment for why that's a safe, fully-functional default rather
/// than a gap to fill in.
pub fn cors_layer(allowed_origins: &[String]) -> Option<tower_http::cors::CorsLayer> {
    if allowed_origins.is_empty() {
        return None;
    }
    let origins: Vec<HeaderValue> = allowed_origins.iter().filter_map(|o| o.parse().ok()).collect();
    Some(
        tower_http::cors::CorsLayer::new()
            .allow_origin(tower_http::cors::AllowOrigin::list(origins))
            .allow_credentials(true)
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([axum::http::header::CONTENT_TYPE]),
    )
}
