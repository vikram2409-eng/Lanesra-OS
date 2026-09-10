//! AI & Agentic Layer, Phase 7a/7c: a small, real PII/DLP detector - built
//! now, in 7a, because the Gateway's forced-air-gap routing
//! (`ai_gateway_service::dispatch`) needs it to inspect an outbound
//! dispatch's payload against an agent's `AiAgentModelRouting::
//! force_air_gapped_for` list before Phase 7c's own Guardrails engine
//! exists to run a post-execution redaction pass. Both reuse this same
//! module - `scan` for the Gateway's pre-dispatch check now, `redact` for
//! 7c's post-execution scrub of a tool result/final answer, generalizing
//! the exact "redact before persistence" convention Phase 5 already
//! established for connector/API-client secrets.
//!
//! Regex-based, not a trained classifier - the same honest tradeoff every
//! lightweight DLP scanner at this scale makes: real detection for the
//! well-structured entity classes below, not a claim of catching every
//! possible PII shape.

use regex::Regex;

/// The full recognized vocabulary - what an agent's `force_air_gapped_for`
/// list, and later 7c's Guardrails config, may name. Kept as a plain
/// `&str` list (not an enum) so it round-trips through JSON exactly like
/// every other free-form tag/name list in this codebase (e.g.
/// `action_names`).
pub const CLASSES: &[&str] = &["ssn", "credit_card", "bank_account", "phone", "email"];

fn patterns() -> Vec<(&'static str, Regex)> {
    vec![
        ("ssn", Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()),
        ("credit_card", Regex::new(r"\b(?:\d[ -]?){13,16}\d\b").unwrap()),
        // Deliberately narrower than a bare 9-digit run (which would
        // double-count SSNs) - a routing/account number formatted with
        // the "acct#"/"routing" style separators real forms and exports
        // use.
        ("bank_account", Regex::new(r"\b\d{9,17}\b").unwrap()),
        ("phone", Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap()),
        ("email", Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap()),
    ]
}

/// Every sensitive-entity class found anywhere in `text` - the Gateway
/// calls this once per dispatch against the full outbound payload
/// (system prompt + latest turn), not per-tool-call, since forced air-
/// gapping is an all-or-nothing routing decision for the whole request.
pub fn scan(text: &str) -> Vec<&'static str> {
    patterns().into_iter().filter(|(_, re)| re.is_match(text)).map(|(class, _)| class).collect()
}

/// Replaces every match of every class with `[REDACTED:<class>]` - used
/// by 7c's post-execution guardrail pass, not by the Gateway's own
/// pre-dispatch `scan` above (which only needs to know *whether* a class
/// matched, not to alter the payload it's about to send).
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    for (class, re) in patterns() {
        out = re.replace_all(&out, format!("[REDACTED:{class}]").as_str()).into_owned();
    }
    out
}
