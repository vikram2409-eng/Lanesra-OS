//! Prompt catalog registry (ADR-0011).
//!
//! Platform prompts live as versioned Markdown files under
//! `desktop/core/prompts/<area>/<name>.v<N>.md`. They are compiled into
//! the binary via `include_str!` so desktop works offline and the files
//! cannot drift from the code; `tests/prompt_catalog.rs` fails if a file on
//! disk is missing from `FILES` or the other way round.
//!
//! Each file starts with YAML front matter (id, version, status, variables,
//! output contract, evaluation suite, changelog). Code asks for the active
//! version of an id and renders it strictly: a missing required variable is
//! an error, and so is an unknown placeholder or an undeclared variable.
//! A prompt is never edited in place once released: a change is a new
//! version file, and `prompts/catalog.lock` pins each released body.

use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

/// Every catalog file, as (path relative to `prompts/`, contents).
pub const FILES: &[(&str, &str)] = &[
    (
        "assistant/admin.v1.md",
        include_str!("../prompts/assistant/admin.v1.md"),
    ),
    (
        "assistant/records.v1.md",
        include_str!("../prompts/assistant/records.v1.md"),
    ),
    (
        "assistant/report-directive.v1.md",
        include_str!("../prompts/assistant/report-directive.v1.md"),
    ),
    (
        "eval/judge.v1.md",
        include_str!("../prompts/eval/judge.v1.md"),
    ),
    (
        "loop/guardrails-section.v1.md",
        include_str!("../prompts/loop/guardrails-section.v1.md"),
    ),
    (
        "loop/json-tool-format.v1.md",
        include_str!("../prompts/loop/json-tool-format.v1.md"),
    ),
    (
        "loop/memory-section.v1.md",
        include_str!("../prompts/loop/memory-section.v1.md"),
    ),
    (
        "loop/session-memory-section.v1.md",
        include_str!("../prompts/loop/session-memory-section.v1.md"),
    ),
    (
        "loop/skills-section.v1.md",
        include_str!("../prompts/loop/skills-section.v1.md"),
    ),
    (
        "voice/planner.v1.md",
        include_str!("../prompts/voice/planner.v1.md"),
    ),
    (
        "voice/planner.v2.md",
        include_str!("../prompts/voice/planner.v2.md"),
    ),
];

/// One declared variable.
#[derive(Debug, Clone, Deserialize)]
pub struct VariableSpec {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default = "default_true")]
    pub required: bool,
    /// Content from outside the platform (a model's answer, a user's text).
    /// Fencing it before insertion is WP-0145; until then it is marked here.
    #[serde(default)]
    pub untrusted: bool,
}

fn default_true() -> bool {
    true
}

/// A prompt file's front matter.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
    pub id: String,
    pub version: u32,
    /// `draft`, `active` or `retired`.
    pub status: String,
    pub work_package: String,
    #[serde(default)]
    pub variables: BTreeMap<String, VariableSpec>,
    pub output_contract: String,
    pub eval_suite: String,
    pub changelog: String,
}

/// A parsed prompt file: front matter + body template.
#[derive(Debug, Clone)]
pub struct Prompt {
    pub id: &'static str,
    pub version: u32,
    /// The whole file, front matter included.
    pub body: &'static str,
    pub path: &'static str,
    pub front: FrontMatter,
}

/// Which prompt version a model call used, for its trace (ADR-0011 §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptRef {
    pub id: String,
    pub version: u32,
}

impl Prompt {
    /// The template text after the front matter.
    pub fn template(&self) -> &'static str {
        body_only(self.body)
    }

    pub fn reference(&self) -> PromptRef {
        PromptRef {
            id: self.id.to_string(),
            version: self.version,
        }
    }
}

/// Parses a file's front matter. Errors name the problem.
pub fn front_matter(full_text: &str) -> Result<FrontMatter, String> {
    let rest = full_text
        .strip_prefix("---\n")
        .ok_or("the file does not start with front matter (---)")?;
    let end = rest
        .find("\n---")
        .ok_or("the front matter has no closing ---")?;
    serde_yaml::from_str(&rest[..end]).map_err(|e| format!("front matter: {e}"))
}

static CATALOG: std::sync::OnceLock<Vec<Prompt>> = std::sync::OnceLock::new();

/// Every prompt in the catalog, parsed once. A file whose front matter does
/// not parse is left out here; `tests/prompt_catalog.rs` reports it.
pub fn all() -> &'static [Prompt] {
    CATALOG.get_or_init(|| {
        FILES
            .iter()
            .filter_map(|(path, text)| {
                let front = front_matter(text).ok()?;
                // Parsed once per process, so this is a one-time allocation.
                let id: &'static str = Box::leak(front.id.clone().into_boxed_str());
                Some(Prompt {
                    id,
                    version: front.version,
                    body: text,
                    path,
                    front,
                })
            })
            .collect()
    })
}

/// Get a prompt by id and version. Returns None if not found.
pub fn get(id: &str, version: u32) -> Option<Prompt> {
    all()
        .iter()
        .find(|p| p.id == id && p.version == version)
        .cloned()
}

/// The active version of a prompt: the highest version whose status is
/// `active`. Drafts and retired versions are never picked.
pub fn get_active(id: &str) -> Option<Prompt> {
    all()
        .iter()
        .filter(|p| p.id == id && p.front.status == "active")
        .max_by_key(|p| p.version)
        .cloned()
}

/// Renders the active version of `id` with `vars`, checking them against
/// the declared variables: every required variable must be given, and no
/// undeclared one may be.
pub fn render_active(id: &str, vars: &[(&str, &str)]) -> Result<(String, PromptRef), String> {
    let prompt = get_active(id).ok_or_else(|| format!("prompt catalog: {id} not found"))?;
    let declared = &prompt.front.variables;
    for (name, _) in vars {
        if !declared.contains_key(*name) {
            return Err(format!("prompt {id}: '{name}' is not a declared variable"));
        }
    }
    for (name, spec) in declared {
        if spec.required && !vars.iter().any(|(n, _)| n == name) {
            return Err(format!(
                "prompt {id}: required variable '{name}' is missing"
            ));
        }
    }
    let map: HashMap<&str, &str> = vars.iter().copied().collect();
    let text = render(prompt.template(), &map)?;
    Ok((text, prompt.reference()))
}

/// The placeholder names a template uses, in order of first use.
pub fn placeholders(body: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find("{{") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("}}") else { break };
        let name = after[..close].trim().to_string();
        if !names.contains(&name) {
            names.push(name);
        }
        rest = &after[close + 2..];
    }
    names
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
        let close = rest
            .find("}}")
            .ok_or_else(|| format!("unclosed placeholder starting at byte {abs_open}"))?;
        let var_name = rest[..close].trim();
        let value = vars
            .get(var_name)
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
    fn get_active_returns_highest_active_version() {
        let p = get_active("eval/judge").expect("should exist");
        assert_eq!(p.version, 1);
        // voice/planner v1 is retired; v2 is the active one.
        assert_eq!(get_active("voice/planner").unwrap().version, 2);
        assert_eq!(get("voice/planner", 1).unwrap().front.status, "retired");
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

    #[test]
    fn render_active_checks_declared_variables() {
        let (text, used) = render_active(
            "eval/judge",
            &[("task", "t"), ("criteria", "c"), ("response", "r")],
        )
        .unwrap();
        assert!(text.contains("Success criteria:\nc"));
        assert_eq!(
            used,
            PromptRef {
                id: "eval/judge".into(),
                version: 1
            }
        );
        assert!(render_active("eval/judge", &[("task", "t"), ("criteria", "c")]).is_err());
        assert!(render_active(
            "eval/judge",
            &[
                ("task", "t"),
                ("criteria", "c"),
                ("response", "r"),
                ("extra", "x")
            ]
        )
        .is_err());
    }
}
