//! ADR-0018: which address counts as the caller of the API, and whether an
//! API client's allowed network admits it.
//!
//! The caller is the TCP peer. `X-Forwarded-For` is read only when the peer
//! is itself a trusted proxy (`LANESRA_TRUSTED_PROXIES`), and then walked
//! from the right, skipping trusted proxies, so addresses a caller
//! prepended to the header are never believed. No other header is read.
//! Pure functions: no sockets, no database.

use std::net::IpAddr;

use crate::domain::{AppError, AppResult};
use crate::services::ssrf_guard::AllowEntry;

/// Operator setting: comma- or space-separated IPs and CIDR blocks of the
/// reverse proxies in front of the server. Default empty (trust none).
pub const TRUSTED_PROXIES_ENV: &str = "LANESRA_TRUSTED_PROXIES";

/// Parses a comma- or whitespace-separated list of addresses and CIDR
/// blocks. Empty entries are ignored. The error names the offending entry.
pub fn parse_list(raw: &str) -> Result<Vec<AllowEntry>, String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|entry| !entry.is_empty())
        .map(AllowEntry::parse)
        .collect()
}

/// Reads `LANESRA_TRUSTED_PROXIES`. A malformed entry is an error so the
/// server can refuse to start rather than guess.
pub fn trusted_proxies_from_env() -> Result<Vec<AllowEntry>, String> {
    match std::env::var(TRUSTED_PROXIES_ENV) {
        Ok(raw) => parse_list(&raw),
        Err(_) => Ok(Vec::new()),
    }
}

fn normalise(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

fn is_trusted(proxies: &[AllowEntry], ip: IpAddr) -> bool {
    proxies.iter().any(|entry| entry.contains(ip))
}

fn parse_hop(hop: &str) -> Option<IpAddr> {
    hop.trim()
        .trim_matches(|c| c == '[' || c == ']')
        .parse::<IpAddr>()
        .ok()
        .map(normalise)
}

/// The caller's address. `forwarded_for` is the joined value of every
/// `X-Forwarded-For` header on the request, if any.
pub fn resolve_caller(
    peer: IpAddr,
    forwarded_for: Option<&str>,
    trusted_proxies: &[AllowEntry],
) -> IpAddr {
    let peer = normalise(peer);
    if !is_trusted(trusted_proxies, peer) {
        return peer;
    }
    let Some(header) = forwarded_for else {
        return peer;
    };
    // Right to left: the first hop that is not one of our own proxies is
    // the client. An unparsable hop means the chain cannot be trusted, so
    // fall back to the peer instead of skipping it.
    for hop in header.rsplit(',') {
        match parse_hop(hop) {
            Some(addr) if is_trusted(trusted_proxies, addr) => continue,
            Some(addr) => return addr,
            None => return peer,
        }
    }
    peer
}

/// Whether an API client's `allowed_cidr` restricts anything. A value that
/// does not parse counts as restricted, so it fails closed.
pub fn is_restricted(allowed_cidr: &str) -> bool {
    parse_list(allowed_cidr).map_or(true, |entries| !entries.is_empty())
}

/// Whether an API client's `allowed_cidr` admits the caller. Empty or
/// blank means unrestricted. A stored value that no longer parses admits
/// nobody (fail closed); `validate_allowed_cidr` stops one being saved.
pub fn client_admits(allowed_cidr: &str, caller: IpAddr) -> bool {
    match parse_list(allowed_cidr) {
        Ok(entries) if entries.is_empty() => true,
        Ok(entries) => entries.iter().any(|entry| entry.contains(caller)),
        Err(_) => false,
    }
}

/// Validation for the value an admin types into the Allowed CIDR field.
pub fn validate_allowed_cidr(raw: &str) -> AppResult<()> {
    parse_list(raw)
        .map(|_| ())
        .map_err(|e| AppError::Validation(format!("Allowed CIDR: {e}")))
}
