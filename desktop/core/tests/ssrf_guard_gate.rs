//! A build without `test-support` (how a release binary sees it) treats a
//! process that never declared its edition as Team Workspace: internal
//! addresses are refused, so a missing declaration fails closed (WP-0143,
//! threat T-13). CI runs this file without the feature; with the feature the
//! default is Desktop and the test below does not compile in.
//!
//! Covers: REQ-TOOL-007 (WP-0143)

#[cfg(not(feature = "test-support"))]
#[test]
fn an_undeclared_process_fails_closed() {
    // Covers: REQ-TOOL-007 (WP-0143)
    use lanesra_core::services::ssrf_guard::{self, NetworkPolicy};
    assert_eq!(ssrf_guard::current(), NetworkPolicy::workspace(Vec::new()));
    let err = ssrf_guard::check_url("http://127.0.0.1:11434/v1").unwrap_err();
    assert_eq!(err.code(), Some("policy.network_blocked"));
}
