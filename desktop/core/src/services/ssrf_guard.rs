//! SSRF protection for admin-configured outbound URLs.
//!
//! WP-0143 (T-13, R-27): Every admin-configured outbound URL — OpenAI-compatible
//! base URL, REST/webhook connectors, MCP endpoints, OData — is checked before
//! save and at connect time.
//!
//! Design (owner-approved 2026-10-18):
//! 1. Always blocked, no override: cloud metadata (169.254.169.254,
//!    fd00:ec2::254) and link-local (169.254.0.0/16, fe80::/10).
//! 2. Edition-dependent: Desktop allows loopback/private by default (user's
//!    machine, Ollama, on-prem tools). Team Workspace blocks them by default.
//! 3. Admin allow-list (host:port or CIDR) permits specific entries; changes audited.
//! 4. The resolved IP is checked at connect time, not just the URL string,
//!    to defeat DNS rebinding.

use std::net::{IpAddr, ToSocketAddrs};

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::feature_flag::Edition;

/// Cloud metadata endpoints — always blocked, no override.
fn is_metadata_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            // 169.254.169.254 (AWS, GCP, Azure IMDS)
            o == [169, 254, 169, 254]
        }
        IpAddr::V6(v6) => {
            // fd00:ec2::254 (AWS IMDSv2 IPv6)
            v6.segments() == [0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254]
        }
    }
}

/// Link-local addresses — always blocked, no override.
fn is_link_local(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 169 && o[1] == 254
        }
        IpAddr::V6(v6) => {
            // fe80::/10
            v6.segments()[0] & 0xffc0 == 0xfe80
        }
    }
}

/// Loopback addresses.
fn is_loopback(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.octets()[0] == 127,
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

/// Private addresses (RFC 1918 for v4, fc00::/7 for v6).
fn is_private(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            // 10.0.0.0/8
            if o[0] == 10 {
                return true;
            }
            // 172.16.0.0/12
            if o[0] == 172 && (16..=31).contains(&o[1]) {
                return true;
            }
            // 192.168.0.0/16
            if o[0] == 192 && o[1] == 168 {
                return true;
            }
            false
        }
        IpAddr::V6(v6) => {
            // fc00::/7 (unique local)
            v6.segments()[0] & 0xfe00 == 0xfc00
        }
    }
}

/// Checks if an IP is in the admin allow-list.
/// Supports exact IP match and CIDR notation.
fn is_allow_listed(ip: &IpAddr, allow_list: &[String]) -> bool {
    let ip_str = ip.to_string();
    for entry in allow_list {
        let entry = entry.trim();
        // Exact match (IP or host:port)
        if entry == ip_str {
            return true;
        }
        // host:port format — extract host part
        if let Some(host) = entry.split(':').next() {
            if host == ip_str {
                return true;
            }
        }
        // CIDR notation (basic v4 support)
        if entry.contains('/') {
            if let Some((base, prefix_str)) = entry.split_once('/') {
                if let (Ok(base_ip), Ok(prefix)) =
                    (base.parse::<IpAddr>(), prefix_str.parse::<u8>())
                {
                    if let (IpAddr::V4(base_v4), IpAddr::V4(ip_v4)) = (base_ip, *ip) {
                        let mask = !((1u32 << (32 - prefix)) - 1);
                        if u32::from(base_v4) & mask == u32::from(ip_v4) & mask {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Validates that a URL is safe for outbound connection.
///
/// # Arguments
/// * `url` - The URL to validate
/// * `field_name` - Human-readable field name for errors
/// * `edition` - Desktop allows loopback/private; Workspace blocks by default
/// * `allow_list` - Admin-approved host:port or CIDR entries (audited)
///
/// # Errors
/// Returns `AppError::Classified` with `policy.network_blocked` if the URL
/// resolves to a blocked address.
pub fn validate_outbound_url(
    url: &str,
    field_name: &str,
    edition: Edition,
    allow_list: &[String],
) -> AppResult<()> {
    let url = url.trim();
    if url.is_empty() {
        return Ok(());
    }

    let host = extract_host(url).ok_or_else(|| AppError::Classified {
        class: ErrorClass::Policy,
        code: "policy.network_blocked",
        message: format!("Invalid {}: cannot parse hostname", field_name),
        retry_after_ms: None,
        http_status: None,
    })?;

    // Resolve hostname to IPs
    let addrs = format!("{}:443", host)
        .to_socket_addrs()
        .map_err(|_| AppError::Classified {
            class: ErrorClass::Policy,
            code: "policy.network_blocked",
            message: format!(
                "Invalid {}: cannot resolve hostname '{}'",
                field_name, host
            ),
            retry_after_ms: None,
            http_status: None,
        })?;

    for addr in addrs {
        let ip = addr.ip();

        // Always blocked: metadata and link-local (no override)
        if is_metadata_ip(&ip) || is_link_local(&ip) {
            return Err(AppError::Classified {
                class: ErrorClass::Policy,
                code: "policy.network_blocked",
                message: format!(
                    "Blocked {}: '{}' resolves to a restricted address ({}). \
                    Cloud metadata and link-local addresses are never allowed.",
                    field_name, host, ip
                ),
                retry_after_ms: None,
                http_status: None,
            });
        }

        // Edition-dependent: loopback and private
        let needs_allow_list = match edition {
            Edition::Desktop => false, // Allowed by default on desktop
            Edition::Workspace => is_loopback(&ip) || is_private(&ip),
        };

        if needs_allow_list && !is_allow_listed(&ip, allow_list) {
            return Err(AppError::Classified {
                class: ErrorClass::Policy,
                code: "policy.network_blocked",
                message: format!(
                    "Blocked {}: '{}' resolves to a restricted address ({}). \
                    On Team Workspace, loopback and private addresses require \
                    an admin allow-list entry.",
                    field_name, host, ip
                ),
                retry_after_ms: None,
                http_status: None,
            });
        }
    }

    Ok(())
}

/// Extracts the hostname from a URL string.
fn extract_host(url: &str) -> Option<String> {
    let url = url.trim();
    let after_scheme = if let Some(idx) = url.find("://") {
        &url[idx + 3..]
    } else {
        url
    };
    let host_port = after_scheme
        .split('/')
        .next()?
        .split('?')
        .next()?
        .split('#')
        .next()?;
    let host = if host_port.starts_with('[') {
        host_port
            .split(']')
            .next()?
            .trim_start_matches('[')
            .to_string()
    } else {
        host_port.split(':').next()?.to_string()
    };
    if host.is_empty() {
        None
    } else {
        Some(host.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_blocked_in_both_editions() {
        for edition in [Edition::Desktop, Edition::Workspace] {
            let result = validate_outbound_url(
                "http://169.254.169.254/latest/meta-data/",
                "base URL",
                edition,
                &[],
            );
            assert!(result.is_err(), "metadata should be blocked in {:?}", edition);
        }
    }

    #[test]
    fn link_local_blocked_in_both_editions() {
        for edition in [Edition::Desktop, Edition::Workspace] {
            let result =
                validate_outbound_url("http://169.254.10.20/api", "base URL", edition, &[]);
            assert!(result.is_err(), "link-local blocked in {:?}", edition);
        }
    }

    #[test]
    fn loopback_allowed_on_desktop() {
        let result =
            validate_outbound_url("http://127.0.0.1:8080/api", "base URL", Edition::Desktop, &[]);
        assert!(result.is_ok(), "desktop allows loopback");
    }

    #[test]
    fn loopback_blocked_on_workspace_without_allow_list() {
        let result = validate_outbound_url(
            "http://127.0.0.1:8080/api",
            "base URL",
            Edition::Workspace,
            &[],
        );
        assert!(result.is_err(), "workspace blocks loopback without allow-list");
        let err = format!("{:?}", result.unwrap_err());
        assert!(err.contains("policy.network_blocked"));
    }

    #[test]
    fn loopback_allowed_on_workspace_with_allow_list() {
        let allow = vec!["127.0.0.1:8080".to_string()];
        let result = validate_outbound_url(
            "http://127.0.0.1:8080/api",
            "base URL",
            Edition::Workspace,
            &allow,
        );
        assert!(result.is_ok(), "allow-list permits loopback on workspace");
    }

    #[test]
    fn private_blocked_on_workspace() {
        let result =
            validate_outbound_url("http://192.168.1.1/api", "base URL", Edition::Workspace, &[]);
        assert!(result.is_err(), "workspace blocks private");
    }

    #[test]
    fn private_allowed_on_desktop() {
        let result =
            validate_outbound_url("http://192.168.1.1/api", "base URL", Edition::Desktop, &[]);
        assert!(result.is_ok(), "desktop allows private");
    }
}
