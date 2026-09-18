//! Voice-First Mode, PR 1: resolves a spoken record reference to an actual
//! record id (spec §8/§8.1), reusing `entity_registry` (generic record
//! existence/display) and `search_service::global_search` (the same
//! substring match across every core entity's natural display fields -
//! including each entity's own record-number field - plus active custom
//! objects) rather than a second lookup mechanism. This is what makes
//! VOICE-AC-06 true for resolution, not just for field metadata: a custom
//! object is searchable/resolvable the moment it exists, with no new code.
//!
//! Implements spec §8.1's priority order as: current-context short-circuit
//! first, then everything else folds into "how many of `global_search`'s
//! hits are plausible" - one exact-title hit is an explicit identifier or
//! an exact name (they look the same from here: `global_search` already
//! matches on a record's own number field), more than one is always a
//! clarification, never a silent guess (VOICE-AC-05). When the substring
//! match finds nothing at all, `search_service::fuzzy_search` provides one
//! more, lower-confidence tier - a bounded edit-distance pass that
//! tolerates a typo - before finally giving up.

use rusqlite::Connection;

use crate::domain::AppResult;
use crate::models::voice::ResolutionCandidate;
use crate::services::{entity_registry, search_service};

#[derive(Debug, Clone)]
pub enum ResolutionOutcome {
    Resolved { record_id: String, object_key: String, confidence: f64 },
    NeedsClarification { candidates: Vec<ResolutionCandidate> },
    NotFound,
}

fn is_context_reference(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    matches!(t.as_str(), "" | "this" | "it" | "that" | "this record" | "this one" | "the current one" | "current record")
}

#[allow(clippy::too_many_arguments)]
pub fn resolve_by_reference(
    conn: &Connection,
    workspace_id: &str,
    object_key_hint: Option<&str>,
    reference_text: &str,
    context_object_key: Option<&str>,
    context_record_id: Option<&str>,
    conversation_reference: Option<(&str, &str)>,
) -> AppResult<ResolutionOutcome> {
    let text = reference_text.trim();

    // Priority: current record / selected record context (spec §7/§8.1) -
    // "this", "it", or simply nothing said beyond the verb ("mark this
    // Won" already consumed the reference as empty by the time it gets
    // here). Confidence 1.0: the user is looking straight at the record.
    if is_context_reference(text) {
        if let (Some(ck), Some(cid)) = (context_object_key, context_record_id) {
            if object_key_hint.map(|h| h.eq_ignore_ascii_case(ck)).unwrap_or(true) && entity_registry::exists(conn, ck, cid)? {
                return Ok(ResolutionOutcome::Resolved { record_id: cid.to_string(), object_key: ck.to_string(), confidence: 1.0 });
            }
        }
        // Second priority (spec §14, PR 2): no record on screen matched,
        // but "it"/"that" may still refer to whatever this session's own
        // bounded conversation history last resolved a few turns back -
        // real multi-turn context, not just the record currently open.
        // Slightly lower confidence than a live on-screen record, since
        // the user isn't looking straight at it anymore.
        if let Some((ck, cid)) = conversation_reference {
            if object_key_hint.map(|h| h.eq_ignore_ascii_case(ck)).unwrap_or(true) && entity_registry::exists(conn, ck, cid)? {
                return Ok(ResolutionOutcome::Resolved { record_id: cid.to_string(), object_key: ck.to_string(), confidence: 0.9 });
            }
        }
        if text.is_empty() {
            return Ok(ResolutionOutcome::NotFound);
        }
    }

    let all_results = search_service::global_search(conn, workspace_id, text)?;
    let results: Vec<_> = match object_key_hint {
        // A hint narrows strictly - if the user (or the planner) named an
        // object type and nothing of that type matched, that's a real
        // "not found", not a silent hand-off to some other object type
        // that happened to match the same words.
        Some(hint) => all_results.into_iter().filter(|r| r.entity_type.eq_ignore_ascii_case(hint)).collect(),
        None => all_results,
    };

    if results.is_empty() {
        // Substring match found nothing at all - fall back to a bounded,
        // typo-tolerant edit-distance pass (spec §8.1's own named "fuzzy
        // match" tier) before giving up. Still never silently guesses: one
        // good fuzzy hit resolves (at a lower confidence than an exact or
        // substring match), more than one always asks (VOICE-AC-05).
        let all_fuzzy = search_service::fuzzy_search(conn, workspace_id, text)?;
        let fuzzy: Vec<_> = match object_key_hint {
            Some(hint) => all_fuzzy.into_iter().filter(|r| r.entity_type.eq_ignore_ascii_case(hint)).collect(),
            None => all_fuzzy,
        };
        if fuzzy.is_empty() {
            return Ok(ResolutionOutcome::NotFound);
        }
        if fuzzy.len() == 1 {
            let r = &fuzzy[0];
            return Ok(ResolutionOutcome::Resolved { record_id: r.entity_id.clone(), object_key: r.entity_type.clone(), confidence: 0.5 });
        }
        let candidates = fuzzy
            .iter()
            .take(6)
            .map(|r| ResolutionCandidate {
                record_id: r.entity_id.clone(),
                label: match &r.subtitle {
                    Some(s) => format!("{} ({} - {})", r.title, r.entity_type, s),
                    None => format!("{} ({})", r.title, r.entity_type),
                },
            })
            .collect();
        return Ok(ResolutionOutcome::NeedsClarification { candidates });
    }

    let exact: Vec<_> = results.iter().filter(|r| r.title.eq_ignore_ascii_case(text)).collect();
    if exact.len() == 1 {
        let r = exact[0];
        return Ok(ResolutionOutcome::Resolved { record_id: r.entity_id.clone(), object_key: r.entity_type.clone(), confidence: 0.95 });
    }
    if exact.is_empty() && results.len() == 1 {
        // A single fuzzy (substring, not exact-title) match - real, but
        // lower confidence than an exact hit; `voice_risk_service`/the
        // planner may still choose to clarify on this depending on the
        // action's own risk, but the resolver itself doesn't force it.
        let r = &results[0];
        return Ok(ResolutionOutcome::Resolved { record_id: r.entity_id.clone(), object_key: r.entity_type.clone(), confidence: 0.7 });
    }

    // Either multiple exact-title matches (two Companies both named
    // "Acme") or multiple plausible fuzzy matches - never silently choose
    // (VOICE-AC-05); surface concise choices for the user to pick from
    // ("the Canada one").
    let candidates = results
        .iter()
        .take(6)
        .map(|r| ResolutionCandidate {
            record_id: r.entity_id.clone(),
            label: match &r.subtitle {
                Some(s) => format!("{} ({} - {})", r.title, r.entity_type, s),
                None => format!("{} ({})", r.title, r.entity_type),
            },
        })
        .collect();
    Ok(ResolutionOutcome::NeedsClarification { candidates })
}
