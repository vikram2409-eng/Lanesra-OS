//! AI Agent Platform v2, Phase 7: Process Lane definitions -
//! `process_lanes`/`process_lane_versions` (migration 0095).
//!
//! A lane is a named business process on one Custom Object, made of ordered
//! stages and steps, defined as data in one JSON Schema
//! (`program/spec/schemas/process-lane.schema.json`, ADR-0024) shared by the
//! online demo, desktop and Team Workspace. A lane is versioned, Draft ->
//! Published, and a run keeps the version it started with through the
//! compiled graph's `source_id` (`{lane_id}@v{version}`, ADR-0024 §3).
//!
//! The typed structs below mirror the schema field for field (snake_case,
//! same optionality); `parse_definition` is the one place a definition JSON
//! becomes a typed lane, and `validate_definition` enforces the schema's
//! kind-conditional rules in Rust (a check step has no role and must have a
//! check; record and approval steps must have writes; branches only on
//! check/approval steps; repeat only on form/record/agent steps) so the
//! core refuses a definition the schema would refuse, without depending on
//! a JSON Schema validator at runtime - the program CI check (WP-0701)
//! validates the same files against the schema itself.

use serde::{Deserialize, Serialize};

use crate::domain::conditions::CONDITION_OPERATORS;
use crate::domain::{AppError, AppResult, ErrorClass};

/// `process_lane_versions.status` - Draft -> Published, never edited in
/// place once published (ADR-0024 §1).
pub const LANE_STATUSES: &[&str] = &["draft", "published"];

/// Step kinds (ADR-0024 §2). Notification is never a kind; it is an
/// automatic effect (`notify` on a step).
pub const LANE_STEP_KINDS: &[&str] = &["form", "record", "check", "approval", "agent"];

/// Agent assistance levels (ADR-0024 §7).
pub const LANE_AUTONOMY_LEVELS: &[&str] = &["person", "propose", "approve", "auto"];

/// The safety bound a compiled repeat loop runs at most: the child count is
/// only known at run time, and the engine's `loop` node needs a positive
/// `max_iterations` at compile time (see `process_lane_service::compile_lane`).
/// WP-0710 (repeat over child records) replaces this bound with the real
/// per-run count; until then a repeating step never runs more than this many
/// iterations.
pub const LANE_REPEAT_MAX_ITERATIONS: i64 = 1000;

fn validation_error(message: impl Into<String>) -> AppError {
    AppError::classified(ErrorClass::Validation, "validation.arguments", message)
}

/// A `$`-reference naming a field: `$record.<field>`, `$run_data.<field>` or
/// `$item.<field>` (inside a repeating step only), with optional `[]` on a
/// collection segment - the same shape the schema's `field` pattern allows.
fn is_field_ref(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('$') else {
        return false;
    };
    let mut parts = rest.split('.');
    match parts.next() {
        Some("record") | Some("run_data") | Some("item") => {}
        _ => return false,
    }
    let mut any = false;
    for part in parts {
        let seg = part.strip_suffix("[]").unwrap_or(part);
        if seg.is_empty()
            || !seg
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return false;
        }
        any = true;
    }
    any
}

fn is_kebab_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLane {
    pub id: String,
    pub workspace_id: String,
    pub lane_key: String,
    pub name: String,
    pub description: Option<String>,
    pub object_key: String,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
    #[serde(default)]
    pub versions: Vec<ProcessLaneVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLaneVersion {
    pub id: String,
    pub lane_id: String,
    pub workspace_id: String,
    pub version: i64,
    pub status: String,
    pub definition_json: String,
    pub graph_id: Option<String>,
    pub created_at: String,
    pub created_by: Option<String>,
    pub updated_at: String,
    pub updated_by: Option<String>,
}

/// The top level of a lane definition file: the format version plus the
/// lane itself (schema `properties.format_version` / `properties.lane`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneDefinitionFile {
    pub format_version: String,
    pub lane: LaneDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub object_key: String,
    pub version: i64,
    pub status: String,
    pub stages: Vec<LaneStage>,
    #[serde(default)]
    pub run_data: Option<RunDataSchema>,
    #[serde(default)]
    pub start_condition: Option<ConditionOrGroup>,
    #[serde(default)]
    pub excludes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDataSchema {
    pub fields: Vec<RunDataField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunDataField {
    pub key: String,
    #[serde(rename = "type")]
    pub field_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub per_child: bool,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneStage {
    pub id: String,
    pub name: String,
    pub steps: Vec<LaneStep>,
    #[serde(default)]
    pub entry_condition: Option<ConditionOrGroup>,
    #[serde(default)]
    pub skip_condition: Option<ConditionOrGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneStep {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default = "default_required")]
    pub required: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assistance: Option<Assistance>,
    #[serde(default)]
    pub fields: Option<Vec<FormField>>,
    #[serde(default)]
    pub writes: Option<Vec<WriteOp>>,
    #[serde(default)]
    pub check: Option<CheckRule>,
    #[serde(default)]
    pub validation: Option<StepValidation>,
    #[serde(default)]
    pub branches: Option<Vec<Branch>>,
    #[serde(default)]
    pub repeat_over: Option<RepeatOver>,
    #[serde(default)]
    pub ends_run: Option<EndsRun>,
    #[serde(default)]
    pub notify: Option<Vec<NotifyRef>>,
    #[serde(default)]
    pub replaces: Option<Vec<ReplacesRef>>,
    #[serde(default)]
    pub child_lane: Option<ChildLane>,
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assistance {
    pub autonomy: String,
    #[serde(default)]
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub binds_to: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteOp {
    pub action_type: String,
    pub inputs: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepValidation {
    #[serde(default)]
    pub condition: Option<ConditionOrGroup>,
    #[serde(default)]
    pub expression: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub on: String,
    #[serde(default)]
    pub target: Option<BranchTarget>,
    #[serde(default)]
    pub end: Option<EndsRun>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchTarget {
    pub stage: String,
    pub step: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndsRun {
    pub outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyRef {
    pub to_role: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacesRef {
    pub workflow_id: String,
    pub action_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatOver {
    pub field: String,
    #[serde(default = "default_item_id_field")]
    pub item_id_field: String,
}

fn default_item_id_field() -> String {
    "id".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildLane {
    pub lane_id: String,
    pub anchor_record: String,
}

/// One check-step rule (ADR-0024 §5): `anchor` evaluates once against the
/// anchor record and run data; `per_item` evaluates once per child record
/// and aggregates with `all` / `any`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRule {
    pub scope: String,
    pub message: String,
    #[serde(default)]
    pub condition: Option<ConditionOrGroup>,
    #[serde(default)]
    pub expression: Option<String>,
    #[serde(default)]
    pub aggregate: Option<String>,
}

/// A condition or a group of conditions, the same language Business Rules
/// use (`domain::conditions::CONDITION_OPERATORS`, match `all` / `any`).
/// A condition compares one `$`-field against a literal `value` or another
/// field (`compare_field`); an `expression` is the narrow sum-comparison
/// grammar of ADR-0024 §5 (evaluation is WP-0710's).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionOrGroup {
    Condition(LaneCondition),
    Group(ConditionGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaneCondition {
    pub field: String,
    pub operator: String,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub compare_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionGroup {
    #[serde(rename = "match")]
    pub match_type: String,
    pub conditions: Vec<ConditionOrGroup>,
}

/// Parses a definition file and validates it (see `validate_definition`).
/// The definition JSON is stored verbatim in `process_lane_versions`, so a
/// lane imported from the demo (`program/spec/schemas/examples/`) keeps the
/// exact bytes it was authored with.
pub fn parse_definition(definition_json: &str) -> AppResult<LaneDefinitionFile> {
    let file: LaneDefinitionFile = serde_json::from_str(definition_json).map_err(|e| {
        validation_error(format!("lane definition is not valid JSON: {e}"))
    })?;
    validate_definition(&file.lane)?;
    Ok(file)
}

/// The schema's kind-conditional rules, enforced in Rust so the core refuses
/// a definition the JSON Schema would refuse (WP-0701's program CI check
/// validates the same files against the schema itself). Structural shape
/// (required fields, types) is already enforced by deserialization above.
pub fn validate_definition(lane: &LaneDefinition) -> AppResult<()> {
    if !lane.id.starts_with("lane-") || !is_kebab_id(&lane.id[5..]) {
        return Err(validation_error(
            "lane id must start with 'lane-' followed by kebab-case",
        ));
    }
    if lane.name.trim().is_empty() {
        return Err(validation_error("lane name is required"));
    }
    if lane.version < 1 {
        return Err(validation_error("lane version must be at least 1"));
    }
    if !LANE_STATUSES.contains(&lane.status.as_str()) {
        return Err(validation_error(
            "lane status must be 'draft' or 'published'",
        ));
    }
    if lane.stages.is_empty() {
        return Err(validation_error("a lane needs at least one stage"));
    }
    if let Some(excludes) = &lane.excludes {
        for other in excludes {
            if !other.starts_with("lane-") {
                return Err(validation_error(
                    "an excluded lane id must start with 'lane-'",
                ));
            }
        }
    }
    if let Some(condition) = &lane.start_condition {
        validate_condition(condition, "start_condition")?;
    }
    if let Some(run_data) = &lane.run_data {
        let mut seen = std::collections::HashSet::new();
        for field in &run_data.fields {
            if !seen.insert(field.key.as_str()) {
                return Err(validation_error(format!(
                    "run_data declares field '{}' twice",
                    field.key
                )));
            }
            if !matches!(
                field.field_type.as_str(),
                "string" | "integer" | "number" | "boolean" | "date"
            ) {
                return Err(validation_error(format!(
                    "run_data field '{}' has unknown type '{}'",
                    field.key, field.field_type
                )));
            }
        }
    }
    let mut stage_ids = std::collections::HashSet::new();
    for stage in &lane.stages {
        if !stage_ids.insert(stage.id.as_str()) {
            return Err(validation_error(format!(
                "stage id '{}' is used twice",
                stage.id
            )));
        }
        if !is_kebab_id(&stage.id) {
            return Err(validation_error(format!(
                "stage id '{}' must be kebab-case",
                stage.id
            )));
        }
        if stage.steps.is_empty() {
            return Err(validation_error(format!(
                "stage '{}' needs at least one step",
                stage.id
            )));
        }
        if let Some(condition) = &stage.entry_condition {
            validate_condition(condition, "entry_condition")?;
        }
        if let Some(condition) = &stage.skip_condition {
            validate_condition(condition, "skip_condition")?;
        }
        let mut step_ids = std::collections::HashSet::new();
        for step in &stage.steps {
            validate_step(lane, stage, step, &mut step_ids)?;
        }
    }
    // Every branch target must name a real step (a dangling target is a
    // dead end the publish check would otherwise only catch later).
    for stage in &lane.stages {
        for step in &stage.steps {
            if let Some(branches) = &step.branches {
                for branch in branches {
                    if let Some(target) = &branch.target {
                        let found = lane.stages.iter().any(|s| {
                            s.id == target.stage
                                && s.steps.iter().any(|st| st.id == target.step)
                        });
                        if !found {
                            return Err(validation_error(format!(
                                "step '{}' branches to unknown step '{}/{}'",
                                step.id, target.stage, target.step
                            )));
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_step(
    _lane: &LaneDefinition,
    stage: &LaneStage,
    step: &LaneStep,
    step_ids: &mut std::collections::HashSet<String>,
) -> AppResult<()> {
    if !step_ids.insert(step.id.clone()) {
        return Err(validation_error(format!(
            "step id '{}' is used twice in stage '{}'",
            step.id, stage.id
        )));
    }
    if !is_kebab_id(&step.id) {
        return Err(validation_error(format!(
            "step id '{}' must be kebab-case",
            step.id
        )));
    }
    if !LANE_STEP_KINDS.contains(&step.kind.as_str()) {
        return Err(validation_error(format!(
            "step '{}' has unknown kind '{}'",
            step.id, step.kind
        )));
    }
    if step.name.trim().is_empty() {
        return Err(validation_error(format!(
            "step '{}' needs a name",
            step.id
        )));
    }
    // Kind-conditional rules (the schema's if/then, mirrored here).
    match step.kind.as_str() {
        "check" => {
            if step.role.is_some() {
                return Err(validation_error(format!(
                    "check step '{}' must not have a role - it runs automatically",
                    step.id
                )));
            }
        }
        _ => {
            if step.role.as_deref().unwrap_or("").trim().is_empty() {
                return Err(validation_error(format!(
                    "step '{}' needs a role",
                    step.id
                )));
            }
        }
    }
    match step.kind.as_str() {
        "approval" => {
            if step.branches.as_ref().map(|b| b.is_empty()).unwrap_or(true) {
                return Err(validation_error(format!(
                    "approval step '{}' must declare its branches",
                    step.id
                )));
            }
        }
        "check" | "approval" => {}
        _ => {
            if step.branches.is_some() {
                return Err(validation_error(format!(
                    "only check and approval steps declare branches (step '{}')",
                    step.id
                )));
            }
        }
    }
    match step.kind.as_str() {
        "record" | "approval" => {
            if step.writes.as_ref().map(|w| w.is_empty()).unwrap_or(true) {
                return Err(validation_error(format!(
                    "step '{}' must declare its writes",
                    step.id
                )));
            }
        }
        _ => {
            if step.writes.is_some() {
                return Err(validation_error(format!(
                    "only record and approval steps declare writes (step '{}')",
                    step.id
                )));
            }
        }
    }
    if step.kind.as_str() != "form" && step.fields.is_some() {
        return Err(validation_error(format!(
            "only form steps declare fields (step '{}')",
            step.id
        )));
    }
    if step.kind.as_str() == "check" && step.check.is_none() {
        return Err(validation_error(format!(
            "check step '{}' must declare its check rule",
            step.id
        )));
    }
    if step.kind.as_str() != "check" && step.check.is_some() {
        return Err(validation_error(format!(
            "only check steps declare a check rule (step '{}')",
            step.id
        )));
    }
    if let Some(check) = &step.check {
        if !matches!(check.scope.as_str(), "anchor" | "per_item") {
            return Err(validation_error(format!(
                "check step '{}' scope must be 'anchor' or 'per_item'",
                step.id
            )));
        }
        match (check.scope.as_str(), &check.aggregate) {
            ("per_item", None) => {
                return Err(validation_error(format!(
                    "check step '{}' with per_item scope needs an aggregate",
                    step.id
                )));
            }
            ("anchor", Some(_)) => {
                return Err(validation_error(format!(
                    "check step '{}' with anchor scope must not declare an aggregate",
                    step.id
                )));
            }
            _ => {}
        }
        if let Some(aggregate) = &check.aggregate {
            if !matches!(aggregate.as_str(), "all" | "any") {
                return Err(validation_error(format!(
                    "check step '{}' aggregate must be 'all' or 'any'",
                    step.id
                )));
            }
        }
        validate_rule_shape(&check.condition, &check.expression, "check")?;
    }
    if !matches!(step.kind.as_str(), "form" | "record" | "agent") && step.repeat_over.is_some() {
        return Err(validation_error(format!(
            "only form, record and agent steps repeat over child records (step '{}')",
            step.id
        )));
    }
    if let Some(assistance) = &step.assistance {
        if !LANE_AUTONOMY_LEVELS.contains(&assistance.autonomy.as_str()) {
            return Err(validation_error(format!(
                "step '{}' has unknown autonomy level '{}'",
                step.id, assistance.autonomy
            )));
        }
        if assistance.autonomy != "person" && assistance.agent.is_none() {
            return Err(validation_error(format!(
                "step '{}' needs an agent for autonomy '{}'",
                step.id, assistance.autonomy
            )));
        }
    }
    if let Some(fields) = &step.fields {
        for field in fields {
            if !matches!(
                field.field_type.as_str(),
                "string" | "integer" | "number" | "boolean" | "date"
            ) {
                return Err(validation_error(format!(
                    "field '{}' on step '{}' has unknown type '{}'",
                    field.key, step.id, field.field_type
                )));
            }
            if !field.binds_to.starts_with("run_data.") && !field.binds_to.starts_with("record.")
            {
                return Err(validation_error(format!(
                    "field '{}' on step '{}' must bind to run_data.<field> or record.<field>",
                    field.key, step.id
                )));
            }
        }
    }
    if let Some(writes) = &step.writes {
        for write in writes {
            if write.action_type.trim().is_empty() {
                return Err(validation_error(format!(
                    "step '{}' declares a write with no action_type",
                    step.id
                )));
            }
        }
    }
    if let Some(validation) = &step.validation {
        validate_rule_shape(&validation.condition, &validation.expression, "validation")?;
    }
    if let Some(branches) = &step.branches {
        for branch in branches {
            match (&branch.target, &branch.end) {
                (Some(_), Some(_)) => {
                    return Err(validation_error(format!(
                        "branch '{}' on step '{}' must declare exactly one of target or end",
                        branch.on, step.id
                    )));
                }
                (None, None) => {
                    return Err(validation_error(format!(
                        "branch '{}' on step '{}' must declare a target or an end",
                        branch.on, step.id
                    )));
                }
                _ => {}
            }
        }
    }
    if let Some(child_lane) = &step.child_lane {
        if !child_lane.lane_id.starts_with("lane-") {
            return Err(validation_error(format!(
                "child lane on step '{}' must start with 'lane-'",
                step.id
            )));
        }
        if !is_field_ref(&child_lane.anchor_record) {
            return Err(validation_error(format!(
                "child lane on step '{}' has a bad anchor_record reference",
                step.id
            )));
        }
    }
    Ok(())
}

fn validate_rule_shape(
    condition: &Option<ConditionOrGroup>,
    expression: &Option<String>,
    what: &str,
) -> AppResult<()> {
    match (condition, expression) {
        (Some(_), Some(_)) => Err(validation_error(format!(
            "{what} must carry exactly one of condition or expression"
        ))),
        (None, None) => Err(validation_error(format!(
            "{what} must carry a condition or an expression"
        ))),
        (Some(c), None) => validate_condition(c, what),
        (None, Some(e)) => validate_expression(e),
    }
}

/// The narrow sum-comparison grammar (ADR-0024 §5): a sum of `$`-references
/// and numbers, one comparator, one `$`-reference or number. No other
/// expression language is admitted in v1.
fn validate_expression(expression: &str) -> AppResult<()> {
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(validation_error(format!(
            "expression '{expression}' is not a sum comparison"
        )));
    }
    let comparator = parts[parts.len() - 2];
    if !matches!(comparator, "<=" | "<" | ">=" | ">" | "==" | "!=") {
        return Err(validation_error(format!(
            "expression '{expression}' needs one comparator"
        )));
    }
    let mut expect_operand = true;
    for part in &parts[..parts.len() - 2] {
        if expect_operand {
            if !is_field_ref(part) && part.parse::<f64>().is_err() {
                return Err(validation_error(format!(
                    "expression '{expression}' has a bad operand '{part}'"
                )));
            }
            expect_operand = false;
        } else if *part != "+" {
            return Err(validation_error(format!(
                "expression '{expression}' only admits '+' between operands"
            )));
        } else {
            expect_operand = true;
        }
    }
    if expect_operand {
        return Err(validation_error(format!(
            "expression '{expression}' ends with '+'"
        )));
    }
    let rhs = parts[parts.len() - 1];
    if !is_field_ref(rhs) && rhs.parse::<f64>().is_err() {
        return Err(validation_error(format!(
            "expression '{expression}' has a bad right-hand side '{rhs}'"
        )));
    }
    Ok(())
}

fn validate_condition(condition: &ConditionOrGroup, what: &str) -> AppResult<()> {
    match condition {
        ConditionOrGroup::Condition(c) => {
            if !is_field_ref(&c.field) {
                return Err(validation_error(format!(
                    "{what} field '{}' must be a $record, $run_data or $item reference",
                    c.field
                )));
            }
            if !CONDITION_OPERATORS.contains(&c.operator.as_str()) {
                return Err(validation_error(format!(
                    "{what} operator '{}' is not a Business Rules operator",
                    c.operator
                )));
            }
            let valueless = matches!(c.operator.as_str(), "is_empty" | "is_not_empty");
            if valueless && (c.value.is_some() || c.compare_field.is_some()) {
                return Err(validation_error(format!(
                    "{what} operator '{}' takes no value",
                    c.operator
                )));
            }
            if !valueless && c.value.is_none() && c.compare_field.is_none() {
                return Err(validation_error(format!(
                    "{what} needs a value or a compare_field"
                )));
            }
            if let Some(other) = &c.compare_field {
                if !is_field_ref(other) {
                    return Err(validation_error(format!(
                        "{what} compare_field '{other}' must be a $record, $run_data or $item reference"
                    )));
                }
            }
            Ok(())
        }
        ConditionOrGroup::Group(g) => {
            if !matches!(g.match_type.as_str(), "all" | "any") {
                return Err(validation_error(format!(
                    "{what} group match must be 'all' or 'any'"
                )));
            }
            if g.conditions.is_empty() {
                return Err(validation_error(format!(
                    "{what} group needs at least one condition"
                )));
            }
            for inner in &g.conditions {
                validate_condition(inner, what)?;
            }
            Ok(())
        }
    }
}
