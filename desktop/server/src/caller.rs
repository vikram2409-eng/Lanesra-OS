//! ADR-0018: the one place the address of an API caller is worked out. The
//! TCP peer is the caller unless it is a configured trusted proxy
//! (`LANESRA_TRUSTED_PROXIES`), in which case `X-Forwarded-For` is read from
//! the right. No other header is consulted.

use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};

use axum::async_trait;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::header::HeaderName;
use axum::http::request::Parts;

use lanesra_core::services::network_trust::resolve_caller;

use crate::state::SharedState;

/// The resolved caller address, or `None` when the server was not given the
/// connection's peer address (it is served without connect info). A
/// restricted API client is refused when this is `None`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caller(pub Option<IpAddr>);

#[async_trait]
impl FromRequestParts<SharedState> for Caller {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip());
        let forwarded = parts
            .headers
            .get_all(HeaderName::from_static("x-forwarded-for"))
            .iter()
            .filter_map(|v| v.to_str().ok())
            .collect::<Vec<_>>()
            .join(",");
        let forwarded = (!forwarded.is_empty()).then_some(forwarded);
        Ok(Caller(peer.map(|peer| {
            resolve_caller(peer, forwarded.as_deref(), &state.security.trusted_proxies)
        })))
    }
}
