//! Prompt catalog registry (ADR-0011).
//!
//! Platform prompts live as versioned Markdown files under
//! `desktop/core/prompts/<area>/<name>.v<N>.md`. They are compiled into
//! the binary via `include_str!` so desktop works offline and the files
//! cannot drift from the code.
//!
//! Code asks for a prompt by id and version, or by id for the active
//! version. Rendering is strict: a missing required variable is an
//! error, and so is an unknown placeholder.

use std::collections::HashMap;

/// A parsed prompt file: front matter + body template.
#[derive(Debug, Clone)]
pub struct Prompt {
    pub id: &'static str,
    pub version: u32,
    pub body: &'static str,
}

/// Registry of all catalog prompts, keyed by (id, version).
fn registry() -> HashMap<(&'static str, u32), Prompt> {
    let mut m = HashMap::new();
    // eval
    m.insert(
        ("eval/judge", 1),
        Prompt {
            id: "eval/judge",
            version: 1,
            body: include_str!("../prompts/eval/judge.v1.md"),
        },
    );
    // assistant
    m.insert(
        ("assistant/records", 1),
        Prompt {
            id: "assistant/records",
            version: 1,
            body: include_str!("../prompts/assistant/records.v1.md"),
        },
    );
    m.insert(
        ("assistant/admin", 1),
        Prompt {
            id: "assistant/admin",
            version: 1,
            body: include_str!("../prompts/assistant/admin.v1.md"),
        },
    );
    // loop
    m.insert(
        ("loop/memory-section", 1),
        Prompt {
            id: "loop/memory-section",
            version: 1,
            body: include_str!("../prompts/loop/memory-section.v1.md"),
        },
    );
    m.insert(
        ("loop/session-memory-section", 1),
        Prompt {
            id: "loop/session-memory-section",
            version: 1,
            body: include_str!("../prompts/loop/session-memory-section.v1.md"),
        },
    );
    m.insert(
        ("loop/guardrails-section", 1),
        Prompt {
            id: "loop/guardrails-section",
            version: 1,
            body: include_str!("../prompts/loop/guardrails-section.v1.md"),
        },
    );
    m.insert(
        ("loop/skills-section", 1),
        Prompt {
            id: "loop/skills-section",
            version: 1,
            body: include_str!("../prompts/loop/skills-section.v1.md"),
        },
    );
    // voice
    m.insert(
        ("voice/planner", 1),
        Prompt {
            id: "voice/planner",
            version: 1,
            body: include_str!("../prompts/voice/planner.v1.md"),
        },
    );
    m
}

/// Get a prompt by id and version. Returns None if not found.
pub fn get(id: &str, version: u32) -> Option<Prompt> {
    registry().get(&(id, version)).cloned()
}

/// Get the active (highest version) prompt for an id. Returns None if not found.
pub fn get_active(id: &str) -> Option<Prompt> {
    let reg = registry();
    let mut best: Option<Prompt> = None;
    for ((pid, _), prompt) in reg.iter() {
        if *pid == id {
            match &best {
                None => best = Some(prompt.clone()),
                Some(b) if prompt.version > b.version => best = Some(prompt.clone()),
                _ => {}
            }
        }
    }
    best
}

/// Render a prompt body by substituting `{{variable}}` placeholders.
/// Returns Err if a required variable is missing or an unknown placeholder is found.
pub fn render(body: &str, vars: &HashMap<&str, &str>) -> Result<String, String> {
    let mut out = body.to_string();
    // Find all {{var}} placeholders
    let mut start = 0;
    while let Some(open) = out[start..].find("{{") {
        let abs_open = start + open;
        let rest = &out[abs_open + 2..];
        let close = rest.find("}}")
            .ok_or_else(|| format!("unclosed placeholder starting at byte {abs_open}"))?;
        let var_name = rest[..close].trim();
        let value = vars.get(var_name)
            .ok_or_else(|| format!("missing required variable: {var_name}"))?;
        let abs_close = abs_open + 2 + close + 2;
        out.replace_range(abs_open..abs_close, value);
        start = abs_open + value.len();
    }
    Ok(out)
}

/// Extract the body (after front matter) from a prompt file's full text.
pub fn body_only(full_text: &str) -> &str {
    // Front matter is between --- lines at the start
    if let Some(stripped) = full_text.strip_prefix("---") {
        if let Some(end) = stripped.find("\n---") {
            let after = &stripped[end + 4..];
            return after.strip_prefix('\n').unwrap_or(after);
        }
    }
    full_text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_prompt_by_id_and_version() {
        let p = get("eval/judge", 1).expect("should exist");
        assert_eq!(p.id, "eval/judge");
        assert_eq!(p.version, 1);
    }

    #[test]
    fn get_active_returns_highest_version() {
        let p = get_active("eval/judge").expect("should exist");
        assert_eq!(p.version, 1);
    }

    #[test]
    fn render_substitutes_variables() {
        let mut vars = HashMap::new();
        vars.insert("task", "do thing");
        vars.insert("criteria", "thing done");
        vars.insert("response", "did thing");
        let body = "Task: {{task}}, Criteria: {{criteria}}";
        let out = render(body, &vars).unwrap();
        assert_eq!(out, "Task: do thing, Criteria: thing done");
    }

    #[test]
    fn render_fails_on_missing_variable() {
        let vars = HashMap::new();
        let body = "Hello {{name}}";
        assert!(render(body, &vars).is_err());
    }
}
