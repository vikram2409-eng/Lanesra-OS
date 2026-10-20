//! Outbound network guard against server-side request forgery (WP-0143;
//! threat T-13, RAID R-27). Owner-approved design, 2026-10-19:
//!
//! - **Always refused, every edition:** cloud metadata endpoints, link-local,
//!   unspecified, broadcast and multicast addresses, also when written as an
//!   IPv4-mapped or NAT64 IPv6 address.
//! - **Internal addresses** (loopback, private, unique-local, shared 100.64/10):
//!   allowed on Desktop, the user's own machine. Refused on Team Workspace
//!   unless the server operator lists them in `LANESRA_OUTBOUND_ALLOW`.
//! - **Checked at connect time.** Every outbound HTTP client in core is built
//!   with `client_builder()`: its DNS resolver hands reqwest only addresses
//!   that pass, so a name that resolves to a safe address when saved and to
//!   an internal one later (DNS rebinding) is still refused. Literal IP hosts
//!   never reach a resolver, so callers run `check_url` before each request,
//!   and the redirect policy checks every redirect target.
//!
//! The edition is declared once per process by the binary that hosts the
//! routes (`install`): the server declares Team Workspace, the desktop app
//! declares Desktop. A process that declares nothing is treated as Team
//! Workspace, so a missing declaration fails closed. The only exception is
//! a `test-support` build, where the default is Desktop so tests can talk to
//! loopback mock servers; `tests/ssrf_guard_gate.rs` proves a normal build
//! fails closed.
//!
//! The allow-list is the server operator's, for the whole server: network
//! reachability belongs to the host, and a workspace admin on a shared server
//! must not be able to open the host's internal network (T-12). Entries are
//! logged at startup.
//!
//! When an outbound HTTP proxy is configured, the proxy resolves names, so
//! name-based checks are the proxy's job; literal IPs are still checked here.

use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{Arc, OnceLock};

use crate::domain::{AppError, AppResult, ErrorClass};
use crate::models::feature_flag::Edition;

pub const ALLOW_LIST_ENV: &str = "LANESRA_OUTBOUND_ALLOW";
const MAX_REDIRECTS: usize = 10;

/// How far an address may be reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Never reachable: the reason is shown to the admin.
    Forbidden(&'static str),
    /// The host's own network: Desktop only, or allow-listed.
    Internal,
    Public,
}

pub fn classify(ip: IpAddr) -> Reach {
    match ip {
        IpAddr::V4(v4) => classify_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return classify_v4(v4);
            }
            let s = v6.segments();
            // NAT64 (64:ff9b::/96) embeds an IPv4 address.
            if s[..6] == [0x64, 0xff9b, 0, 0, 0, 0] {
                return classify_v4(Ipv4Addr::from((u32::from(s[6]) << 16) | u32::from(s[7])));
            }
            if v6 == Ipv6Addr::new(0xfd00, 0xec2, 0, 0, 0, 0, 0, 0x254) {
                Reach::Forbidden("a cloud metadata endpoint")
            } else if s[0] & 0xffc0 == 0xfe80 {
                Reach::Forbidden("a link-local address")
            } else if v6.is_unspecified() {
                Reach::Forbidden("the unspecified address")
            } else if v6.is_multicast() {
                Reach::Forbidden("a multicast address")
            } else if v6.is_loopback() || s[0] & 0xfe00 == 0xfc00 {
                Reach::Internal
            } else {
                Reach::Public
            }
        }
    }
}

fn classify_v4(v4: Ipv4Addr) -> Reach {
    let o = v4.octets();
    if v4 == Ipv4Addr::new(169, 254, 169, 254) || v4 == Ipv4Addr::new(100, 100, 100, 200) {
        Reach::Forbidden("a cloud metadata endpoint")
    } else if v4.is_link_local() {
        Reach::Forbidden("a link-local address")
    } else if o[0] == 0 {
        Reach::Forbidden("the unspecified address")
    } else if v4.is_broadcast() || v4.is_multicast() {
        Reach::Forbidden("a broadcast or multicast address")
    } else if v4.is_loopback() || v4.is_private() || (o[0] == 100 && (o[1] & 0xc0) == 64) {
        Reach::Internal
    } else {
        Reach::Public
    }
}

/// One allow-list entry: an address, or a CIDR block of either family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowEntry {
    network: IpAddr,
    prefix: u8,
}

impl AllowEntry {
    pub fn parse(entry: &str) -> Result<Self, String> {
        let entry = entry.trim();
        let (addr, prefix) = match entry.split_once('/') {
            Some((a, p)) => (a, Some(p)),
            None => (entry, None),
        };
        let network: IpAddr = addr
            .trim_matches(|c| c == '[' || c == ']')
            .parse()
            .map_err(|_| format!("'{entry}' is not an IP address or CIDR block"))?;
        let max = if network.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(p) => p
                .parse::<u8>()
                .ok()
                .filter(|p| *p <= max)
                .ok_or_else(|| format!("'{entry}' has an invalid prefix length"))?,
            None => max,
        };
        Ok(Self { network, prefix })
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = match ip {
            IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
            v4 => v4,
        };
        match (self.network, ip) {
            (IpAddr::V4(n), IpAddr::V4(a)) => prefix_match(
                u128::from(u32::from(n)),
                u128::from(u32::from(a)),
                self.prefix,
                32,
            ),
            (IpAddr::V6(n), IpAddr::V6(a)) => {
                prefix_match(u128::from(n), u128::from(a), self.prefix, 128)
            }
            _ => false,
        }
    }
}

impl fmt::Display for AllowEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix)
    }
}

fn prefix_match(network: u128, addr: u128, prefix: u8, bits: u32) -> bool {
    let host_bits = bits - u32::from(prefix);
    if host_bits >= bits {
        return true;
    }
    (network >> host_bits) == (addr >> host_bits)
}

/// The outbound rules for this process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkPolicy {
    pub edition: Edition,
    pub allow: Vec<AllowEntry>,
}

impl NetworkPolicy {
    pub fn desktop() -> Self {
        Self {
            edition: Edition::Desktop,
            allow: Vec::new(),
        }
    }

    pub fn workspace(allow: Vec<AllowEntry>) -> Self {
        Self {
            edition: Edition::Workspace,
            allow,
        }
    }

    /// Team Workspace with the operator's `LANESRA_OUTBOUND_ALLOW` list
    /// (comma-separated addresses or CIDR blocks). A malformed entry is an
    /// error, so the server refuses to start rather than guess.
    pub fn workspace_from_env() -> Result<Self, String> {
        let raw = std::env::var(ALLOW_LIST_ENV).unwrap_or_default();
        let allow = raw
            .split(',')
            .filter(|e| !e.trim().is_empty())
            .map(AllowEntry::parse)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self::workspace(allow))
    }

    pub fn check_ip(&self, ip: IpAddr) -> Result<(), Blocked> {
        match classify(ip) {
            Reach::Public => Ok(()),
            Reach::Forbidden(what) => Err(Blocked(format!(
                "{ip} is {what}, which is never reachable from Lanesra"
            ))),
            Reach::Internal => {
                if self.edition == Edition::Desktop || self.allow.iter().any(|e| e.contains(ip)) {
                    Ok(())
                } else {
                    Err(Blocked(format!(
                        "{ip} is on the server's internal network; the server operator must \
                         list it in {ALLOW_LIST_ENV} before Team Workspace can reach it"
                    )))
                }
            }
        }
    }
}

static POLICY: OnceLock<NetworkPolicy> = OnceLock::new();

/// Declares this process's outbound rules. Call once, at startup, before any
/// request. A second call with different rules is refused.
pub fn install(policy: NetworkPolicy) -> Result<(), String> {
    let installed = POLICY.get_or_init(|| policy.clone());
    if *installed == policy {
        Ok(())
    } else {
        Err("The outbound network policy was already set for this process".into())
    }
}

/// The installed policy, or the fail-closed default.
pub fn current() -> NetworkPolicy {
    POLICY.get().cloned().unwrap_or_else(default_policy)
}

#[cfg(not(feature = "test-support"))]
fn default_policy() -> NetworkPolicy {
    NetworkPolicy::workspace(Vec::new())
}

#[cfg(feature = "test-support")]
fn default_policy() -> NetworkPolicy {
    NetworkPolicy::desktop()
}

/// Why a connection was refused. Kept as its own error type so it survives
/// reqwest's error wrapping and can be reported as `policy.network_blocked`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Blocked(pub String);

impl fmt::Display for Blocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Blocked {}

impl From<Blocked> for AppError {
    fn from(b: Blocked) -> Self {
        AppError::classified(ErrorClass::Policy, "policy.network_blocked", b.0)
    }
}

/// Checks a URL's scheme and, when its host is a literal IP, the address.
/// Hostnames are checked when they are resolved (`client_builder`).
pub fn check_url(url: &str) -> AppResult<()> {
    check_url_with(&current(), url).map_err(AppError::from)
}

pub fn check_url_with(policy: &NetworkPolicy, url: &str) -> Result<(), Blocked> {
    let parsed = reqwest::Url::parse(url.trim())
        .map_err(|_| Blocked(format!("'{url}' is not a valid URL")))?;
    check_parsed(policy, &parsed)
}

fn check_parsed(policy: &NetworkPolicy, url: &reqwest::Url) -> Result<(), Blocked> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Blocked(format!(
            "Only http and https URLs are allowed, not '{}'",
            url.scheme()
        )));
    }
    let host = url
        .host_str()
        .ok_or_else(|| Blocked(format!("'{url}' has no host")))?;
    // The URL parser has already normalised IPv4 spellings such as
    // `http://2852039166/` to dotted form.
    match host
        .trim_matches(|c| c == '[' || c == ']')
        .parse::<IpAddr>()
    {
        Ok(ip) => policy.check_ip(ip),
        Err(_) => Ok(()),
    }
}

/// The resolved addresses of `host` that the policy allows. Refuses when
/// none is allowed; a name that also resolves to a refused address is still
/// usable through its allowed ones.
pub async fn resolve_allowed(
    policy: &NetworkPolicy,
    host: &str,
) -> Result<Vec<SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
    let resolved: Vec<SocketAddr> = tokio::net::lookup_host((host, 0)).await?.collect();
    let mut first_refusal = None;
    let allowed: Vec<SocketAddr> = resolved
        .into_iter()
        .filter(|addr| match policy.check_ip(addr.ip()) {
            Ok(()) => true,
            Err(b) => {
                first_refusal.get_or_insert(b);
                false
            }
        })
        .collect();
    if allowed.is_empty() {
        let reason = first_refusal.map_or_else(
            || format!("{host} did not resolve to any address"),
            |b| format!("{host} resolves to {b}"),
        );
        return Err(Box::new(Blocked(reason)));
    }
    Ok(allowed)
}

/// The addresses a non-HTTP connection (SFTP, SMTP) may use for
/// `host:port`. Connect to these, not to the name, so the address checked
/// is the address used.
pub async fn allowed_addrs(host: &str, port: u16) -> AppResult<Vec<SocketAddr>> {
    match resolve_allowed(&current(), host).await {
        Ok(addrs) => Ok(addrs
            .into_iter()
            .map(|a| SocketAddr::new(a.ip(), port))
            .collect()),
        Err(e) => Err(match e.downcast_ref::<Blocked>() {
            Some(b) => AppError::from(b.clone()),
            None => AppError::Validation(format!("Could not resolve {host}: {e}")),
        }),
    }
}

struct GuardedResolver;

impl reqwest::dns::Resolve for GuardedResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let host = name.as_str().to_string();
        Box::pin(async move {
            let addrs = resolve_allowed(&current(), &host).await?;
            Ok(Box::new(addrs.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

/// A reqwest client builder whose connections and redirects obey the
/// installed policy. Every outbound HTTP client in core starts here.
pub fn client_builder() -> reqwest::ClientBuilder {
    reqwest::Client::builder()
        .dns_resolver(Arc::new(GuardedResolver))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= MAX_REDIRECTS {
                return attempt.error(Blocked("Too many redirects".into()));
            }
            match check_parsed(&current(), attempt.url()) {
                Ok(()) => attempt.follow(),
                Err(b) => attempt.error(b),
            }
        }))
}

/// The guard's refusal inside a reqwest error, if that is why it failed.
pub fn blocked_in(error: &reqwest::Error) -> Option<AppError> {
    let mut source: Option<&(dyn std::error::Error + 'static)> = Some(error);
    while let Some(e) = source {
        if let Some(b) = e.downcast_ref::<Blocked>() {
            return Some(AppError::from(b.clone()));
        }
        source = e.source();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn metadata_and_link_local_are_forbidden_in_every_spelling() {
        for s in [
            "169.254.169.254",
            "100.100.100.200",
            "fd00:ec2::254",
            "::ffff:169.254.169.254",
            "64:ff9b::a9fe:a9fe",
            "169.254.10.20",
            "fe80::1",
            "0.0.0.0",
            "::",
            "255.255.255.255",
            "224.0.0.1",
        ] {
            assert!(matches!(classify(ip(s)), Reach::Forbidden(_)), "{s}");
        }
    }

    #[test]
    fn internal_and_public_ranges() {
        for s in [
            "127.0.0.1",
            "::1",
            "10.1.2.3",
            "172.16.0.1",
            "192.168.1.1",
            "fc00::5",
            "100.64.0.1",
            "::ffff:127.0.0.1",
        ] {
            assert_eq!(classify(ip(s)), Reach::Internal, "{s}");
        }
        for s in ["8.8.8.8", "172.32.0.1", "100.128.0.1", "2606:4700::1111"] {
            assert_eq!(classify(ip(s)), Reach::Public, "{s}");
        }
    }

    #[test]
    fn allow_entries_match_addresses_and_blocks() {
        let block = AllowEntry::parse("192.168.1.0/24").unwrap();
        assert!(block.contains(ip("192.168.1.200")));
        assert!(block.contains(ip("::ffff:192.168.1.7")));
        assert!(!block.contains(ip("192.168.2.1")));
        let single = AllowEntry::parse("10.0.0.5").unwrap();
        assert!(single.contains(ip("10.0.0.5")) && !single.contains(ip("10.0.0.6")));
        let v6 = AllowEntry::parse("[fd12::]/16").unwrap();
        assert!(v6.contains(ip("fd12:abcd::1")) && !v6.contains(ip("fd13::1")));
        assert!(AllowEntry::parse("0.0.0.0/0")
            .unwrap()
            .contains(ip("10.0.0.1")));
        for bad in ["10.0.0.0/33", "host.local", "10.0.0.0/x", ""] {
            assert!(AllowEntry::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_allow_list_never_opens_forbidden_addresses() {
        let open = NetworkPolicy::workspace(vec![AllowEntry::parse("0.0.0.0/0").unwrap()]);
        assert!(open.check_ip(ip("169.254.169.254")).is_err());
        assert!(open.check_ip(ip("10.0.0.1")).is_ok());
    }
}
