//! Integration tests for the SSRF guard (WP-0143).
//!
//! Covers: REQ-SEC-013 (WP-0143)
//!
//! The guard blocks cloud metadata and link-local addresses in all editions,
//! and blocks loopback/private ranges on Team Workspace (allowed on Desktop).

use lanesra_core::models::feature_flag::Edition;
use lanesra_core::services::ssrf_guard::validate_outbound_url;

fn assert_blocked(url: &str, edition: Edition) {
    let result = validate_outbound_url(url, "test url", edition, &[]);
    assert!(
        result.is_err(),
        "expected {url} to be blocked on {edition:?}"
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("policy.network_blocked"),
        "expected policy.network_blocked, got: {err}"
    );
}

fn assert_allowed(url: &str, edition: Edition) {
    let result = validate_outbound_url(url, "test url", edition, &[]);
    assert!(
        result.is_ok(),
        "expected {url} to be allowed on {edition:?}, got: {:?}",
        result.unwrap_err()
    );
}

#[test]
fn blocks_metadata_in_both_editions() {
    for edition in [Edition::Desktop, Edition::Workspace] {
        assert_blocked("http://169.254.169.254/latest/meta-data/", edition);
        assert_blocked("http://[fd00:ec2::254]/", edition);
    }
}

#[test]
fn blocks_link_local_in_both_editions() {
    for edition in [Edition::Desktop, Edition::Workspace] {
        assert_blocked("http://169.254.10.20/", edition);
        assert_blocked("http://[fe80::1]/", edition);
    }
}

#[test]
fn allows_loopback_on_desktop() {
    assert_allowed("http://127.0.0.1:11434/", Edition::Desktop);
    assert_allowed("http://localhost:3000/", Edition::Desktop);
    assert_allowed("http://[::1]:8080/", Edition::Desktop);
}

#[test]
fn blocks_loopback_on_workspace_without_allow_list() {
    assert_blocked("http://127.0.0.1:11434/", Edition::Workspace);
    assert_blocked("http://[::1]:8080/", Edition::Workspace);
}

#[test]
fn allows_loopback_on_workspace_with_allow_list() {
    let allow = vec!["127.0.0.1:11434".to_string()];
    let result = validate_outbound_url(
        "http://127.0.0.1:11434/v1/models",
        "test url",
        Edition::Workspace,
        &allow,
    );
    assert!(result.is_ok(), "allow-listed loopback should be allowed");
}

#[test]
fn blocks_private_ranges_on_workspace() {
    for edition in [Edition::Workspace] {
        assert_blocked("http://10.0.0.5/", edition);
        assert_blocked("http://172.16.0.5/", edition);
        assert_blocked("http://192.168.1.5/", edition);
        assert_blocked("http://[fc00::5]/", edition);
    }
}

#[test]
fn allows_private_ranges_on_desktop() {
    assert_allowed("http://192.168.1.5:8080/", Edition::Desktop);
    assert_allowed("http://10.0.0.5/", Edition::Desktop);
}

#[test]
fn allows_public_url_in_both_editions() {
    for edition in [Edition::Desktop, Edition::Workspace] {
        assert_allowed("https://api.anthropic.com/v1/messages", edition);
        assert_allowed("https://example.com/webhook", edition);
    }
}

#[test]
fn allow_list_cidr_matching() {
    let allow = vec!["192.168.1.0/24".to_string()];
    let result = validate_outbound_url(
        "http://192.168.1.100:8080/",
        "test url",
        Edition::Workspace,
        &allow,
    );
    assert!(result.is_ok(), "CIDR allow-listed IP should be allowed");

    // Outside the CIDR should still be blocked
    assert_blocked("http://192.168.2.100:8080/", Edition::Workspace);
}
