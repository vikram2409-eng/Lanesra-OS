//! Integration tests for the WP-0143 SSRF guard.
//!
//! Covers: REQ-TOOL-001 (WP-0143 AC-1)
//!
//! The guard blocks cloud metadata and link-local in every edition,
//! and blocks loopback/private on Team Workspace unless allow-listed.

use lanesra_core::models::feature_flag::Edition;
use lanesra_core::services::ssrf_guard::validate_outbound_url;

#[test]
fn blocks_metadata_ip() {
    let result = validate_outbound_url(
        "http://169.254.169.254/latest/meta-data/",
        "base URL",
        Edition::Workspace,
        &[],
    );
    assert!(result.is_err());
    let err = format!("{:?}", result.unwrap_err());
    assert!(err.contains("policy.network_blocked"));
}

#[test]
fn blocks_metadata_in_desktop_too() {
    let result = validate_outbound_url(
        "http://169.254.169.254/latest/meta-data/",
        "base URL",
        Edition::Desktop,
        &[],
    );
    assert!(result.is_err());
}

#[test]
fn blocks_private_on_workspace() {
    for url in [
        "http://10.0.0.1/api",
        "http://172.16.5.4/api",
        "http://192.168.1.1/api",
    ] {
        let result = validate_outbound_url(url, "base URL", Edition::Workspace, &[]);
        assert!(result.is_err(), "should block {} on workspace", url);
    }
}

#[test]
fn allows_private_on_desktop() {
    let result = validate_outbound_url(
        "http://192.168.1.1:8080/api",
        "base URL",
        Edition::Desktop,
        &[],
    );
    assert!(result.is_ok());
}

#[test]
fn loopback_blocked_on_workspace_without_allow_list() {
    let result = validate_outbound_url(
        "http://127.0.0.1:8080/api",
        "base URL",
        Edition::Workspace,
        &[],
    );
    assert!(result.is_err());
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
    assert!(result.is_ok());
}

#[test]
fn allows_loopback_on_desktop() {
    let result =
        validate_outbound_url("http://127.0.0.1:3000/api", "base URL", Edition::Desktop, &[]);
    assert!(result.is_ok());
}

#[test]
fn allows_public_url_in_both_editions() {
    for edition in [Edition::Desktop, Edition::Workspace] {
        // example.com resolves to a public IP
        let result =
            validate_outbound_url("https://api.example.com/v1", "base URL", edition, &[]);
        // May fail on DNS in sandbox, but must not be a policy block
        if let Err(e) = result {
            let err = format!("{:?}", e);
            assert!(
                !err.contains("policy.network_blocked") || err.contains("cannot resolve"),
                "unexpected policy block: {}",
                err
            );
        }
    }
}
