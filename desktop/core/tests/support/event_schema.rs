//! WP-0112: checks run events against their JSON Schemas in
//! `program/spec/schemas/events/` (ADR-0008 §2). Shared by the core and
//! server tests (`#[path]`).
//!
//! It understands only the keywords those schemas use: `type` (one name or
//! a list), `const`, `enum`, `minimum`, `properties`, `required` and
//! `additionalProperties: false`. A schema using any other keyword fails,
//! so a new keyword is never silently ignored.

use std::path::PathBuf;

use serde_json::Value;

const KNOWN: &[&str] = &[
    "$schema",
    "title",
    "description",
    "type",
    "const",
    "enum",
    "minimum",
    "properties",
    "required",
    "additionalProperties",
];

pub fn schema_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../program/spec/schemas/events")
}

pub fn load(name: &str) -> Value {
    let path = schema_dir().join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap()
}

fn type_matches(name: &str, value: &Value) -> bool {
    match name {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "null" => value.is_null(),
        other => panic!("unknown type '{other}' in a schema"),
    }
}

/// Every way `value` breaks `schema`, as `path: problem` lines.
pub fn errors(schema: &Value, value: &Value, path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(schema) = schema.as_object() else {
        return vec![format!("{path}: schema is not an object")];
    };
    for key in schema.keys() {
        assert!(
            KNOWN.contains(&key.as_str()),
            "schema keyword '{key}' at {path} is not supported by this checker"
        );
    }
    if let Some(t) = schema.get("type") {
        let names: Vec<&str> = match t {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !names.iter().any(|n| type_matches(n, value)) {
            out.push(format!("{path}: expected {names:?}, got {value}"));
            return out;
        }
    }
    if let Some(c) = schema.get("const") {
        if c != value {
            out.push(format!("{path}: expected {c}, got {value}"));
        }
    }
    if let Some(Value::Array(options)) = schema.get("enum") {
        if !options.contains(value) {
            out.push(format!("{path}: {value} is not one of {options:?}"));
        }
    }
    if let (Some(min), Some(n)) = (
        schema.get("minimum").and_then(Value::as_f64),
        value.as_f64(),
    ) {
        if n < min {
            out.push(format!("{path}: {n} is below {min}"));
        }
    }
    if let Some(obj) = value.as_object() {
        let props = schema.get("properties").and_then(Value::as_object);
        if let Some(Value::Array(required)) = schema.get("required") {
            for r in required.iter().filter_map(Value::as_str) {
                if !obj.contains_key(r) {
                    out.push(format!("{path}: missing '{r}'"));
                }
            }
        }
        for (k, v) in obj {
            match props.and_then(|p| p.get(k)) {
                Some(sub) => out.extend(errors(sub, v, &format!("{path}.{k}"))),
                None => {
                    if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
                        out.push(format!("{path}: unexpected field '{k}'"));
                    }
                }
            }
        }
    }
    out
}

/// Panics unless `event` (a serialized `RunEvent`) matches the envelope
/// schema and its type's data schema.
pub fn assert_valid(event: &Value) {
    let mut problems = errors(&load("envelope.v1.json"), event, "event");
    let event_type = event["type"].as_str().unwrap_or_default();
    problems.extend(errors(
        &load(&format!("{event_type}.v1.json")),
        &event["data"],
        "event.data",
    ));
    assert!(problems.is_empty(), "{event}\n{}", problems.join("\n"));
}
