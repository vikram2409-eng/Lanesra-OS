//! WP-0184 / ADR-0018 section 1 and 2: which address counts as the caller of
//! the API, and whether a client's allowed network admits it. Pure logic, no
//! database and no sockets.

use std::net::IpAddr;

use lanesra_core::services::network_trust::{
    client_admits, parse_list, resolve_caller, validate_allowed_cidr,
};

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

fn trusted(list: &str) -> Vec<lanesra_core::services::ssrf_guard::AllowEntry> {
    parse_list(list).unwrap()
}

// Covers: REQ-INT-004 (WP-0184 AC-3)
#[test]
fn with_no_trusted_proxy_every_forwarding_header_is_ignored() {
    let none = trusted("");
    assert_eq!(
        resolve_caller(ip("203.0.113.9"), Some("10.1.1.1"), &none),
        ip("203.0.113.9")
    );
    assert_eq!(
        resolve_caller(ip("203.0.113.9"), Some("10.1.1.1, 10.2.2.2"), &none),
        ip("203.0.113.9")
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-3)
#[test]
fn a_peer_that_is_not_a_trusted_proxy_cannot_forward_an_address() {
    let proxies = trusted("10.0.0.0/8");
    assert_eq!(
        resolve_caller(ip("203.0.113.9"), Some("198.51.100.7"), &proxies),
        ip("203.0.113.9")
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-4)
#[test]
fn behind_a_trusted_proxy_the_forwarded_address_is_the_caller() {
    let proxies = trusted("10.0.0.0/8");
    assert_eq!(
        resolve_caller(ip("10.0.0.5"), Some("198.51.100.7"), &proxies),
        ip("198.51.100.7")
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-4)
#[test]
fn the_right_most_address_that_is_not_a_trusted_proxy_wins() {
    let proxies = trusted("10.0.0.0/8, 192.0.2.0/24");
    // An attacker prepended 6.6.6.6; the proxy chain added the rest.
    assert_eq!(
        resolve_caller(
            ip("10.0.0.5"),
            Some("6.6.6.6, 198.51.100.7, 192.0.2.4, 10.0.0.9"),
            &proxies
        ),
        ip("198.51.100.7")
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-4)
#[test]
fn a_missing_malformed_or_all_trusted_header_falls_back_to_the_peer() {
    let proxies = trusted("10.0.0.0/8");
    let peer = ip("10.0.0.5");
    assert_eq!(resolve_caller(peer, None, &proxies), peer);
    assert_eq!(resolve_caller(peer, Some(""), &proxies), peer);
    assert_eq!(resolve_caller(peer, Some("not-an-address"), &proxies), peer);
    // An unparsable hop on the way in means the chain cannot be trusted.
    assert_eq!(
        resolve_caller(peer, Some("198.51.100.7, garbage"), &proxies),
        peer
    );
    assert_eq!(
        resolve_caller(peer, Some("10.0.0.9, 10.0.0.8"), &proxies),
        peer
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-5)
#[test]
fn ipv6_bracketed_and_ipv4_mapped_addresses_are_understood() {
    let proxies = trusted("10.0.0.0/8, 2001:db8:ffff::/48");
    // A mapped peer matches an IPv4 proxy block.
    assert_eq!(
        resolve_caller(ip("::ffff:10.0.0.5"), Some("198.51.100.7"), &proxies),
        ip("198.51.100.7")
    );
    // The resolved caller is normalised to IPv4.
    assert_eq!(
        resolve_caller(ip("10.0.0.5"), Some("::ffff:198.51.100.7"), &proxies),
        ip("198.51.100.7")
    );
    assert_eq!(
        resolve_caller(ip("2001:db8:ffff::1"), Some("[2001:db8:1::9]"), &proxies),
        ip("2001:db8:1::9")
    );
}

// Covers: REQ-INT-004 (WP-0184 AC-5)
#[test]
fn an_allowed_network_list_matches_blocks_single_addresses_and_both_families() {
    let list = "10.0.0.0/8, 203.0.113.7  2001:db8::/32";
    assert!(client_admits(list, ip("10.200.1.1")));
    assert!(client_admits(list, ip("203.0.113.7")));
    assert!(!client_admits(list, ip("203.0.113.8")));
    assert!(client_admits(list, ip("2001:db8:1::1")));
    assert!(!client_admits(list, ip("2001:db9::1")));
    assert!(!client_admits(list, ip("198.51.100.1")));
    // IPv4-mapped IPv6 callers match IPv4 entries.
    assert!(client_admits(list, ip("::ffff:10.1.1.1")));
    assert!(!client_admits(list, ip("::ffff:198.51.100.1")));
}

// Covers: REQ-INT-004 (WP-0184 AC-2)
#[test]
fn an_empty_allowed_network_admits_everyone() {
    for empty in ["", "   ", " , ,"] {
        assert!(client_admits(empty, ip("198.51.100.1")), "{empty:?}");
    }
}

// Covers: REQ-INT-004 (WP-0184 AC-6)
#[test]
fn a_malformed_list_is_rejected_naming_the_bad_entry() {
    assert!(validate_allowed_cidr("10.0.0.0/8, 203.0.113.7").is_ok());
    assert!(validate_allowed_cidr("").is_ok());
    for bad in [
        "10.0.0.0/33",
        "banana",
        "10.0.0.0/8, 999.1.1.1",
        "10.0.0.0/",
    ] {
        assert!(validate_allowed_cidr(bad).is_err(), "{bad:?}");
    }
    let message = validate_allowed_cidr("10.0.0.0/8, banana")
        .unwrap_err()
        .to_string();
    assert!(message.contains("banana"), "{message}");
    // The trusted-proxy parser uses the same rule and names the entry too.
    let err = parse_list("10.0.0.0/8, nope").unwrap_err();
    assert!(err.contains("nope"), "{err}");
}
