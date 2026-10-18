# ADR-0009: Five error classes decide retry, failover and what the user sees

- Status: Accepted (program owner, 2026-10-16, PR #628)
- Date: 2026-10-16
- Work package: WP-0009
- Requirements: REQ-PRE-007, REQ-ORCH-033, REQ-LOOP-006, REQ-LOOP-010
- Constrains: WP-0121, WP-0123, WP-0124, WP-0132, WP-0141, WP-0146, every provider adapter, and every tool dispatcher

## Context

Today almost every failure is a free-text `AppError::Validation(String)`. All 71 errors raised in `ai_service.rs` are of that kind. The only typed variant for AI work is `PolicyBlocked`.

The AI Gateway moves to the next provider tier on *any* failure (`ai_gateway_service.rs:21-28`). That is the deliberate simplification its own comments describe, and it has two bad effects:

- A request refused by policy, or a malformed request, is resent to a second provider.
- A forced air-gap can fall back to a cloud model (RAID I-17).

REQ-ORCH-033 requires that transient failures retry with backoff and that policy and validation errors never retry. That rule cannot be enforced without typed errors.

## Decision

### 1. The five classes

| Class | Examples | Retry the same call | Fail over to the next tier | Shown to the model | End state if not recovered |
|---|---|---|---|---|---|
| `transient` | Network error, timeout, HTTP 429/500/502/503/504, provider overloaded, SQLite busy | **Yes:** up to 3 attempts, exponential backoff with full jitter, base 500 ms, cap 20 s; honour `Retry-After` | Yes, after the retries | No | Step `failed`, `stop_reason=model_unavailable` (or `error` for tools) |
| `policy` | Firewall deny, approval required, budget used up, data-boundary refusal, ontology scope, record access denied | **Never** | **Never** | Yes, as a refusal it can read and act on ("delete_record is blocked by policy") | Approval or budget: the matching waiting status (ADR-0007). Deny: the round continues |
| `validation` | Bad tool arguments, output fails its schema, guardrail fails, unknown tool | **Never as-is** | **Never** | Yes: the reasons go back so the model can repair, within the guardrail retry budget (default 2, REQ-LOOP-006) | Step `failed`, `stop_reason=guardrail_failed`; follow the `on_fail` edge if present |
| `configuration` | Missing or invalid key, provider disabled, model not found, provider not local while air-gap is forced | **Never** | **Yes**, except under a forced air-gap, which fails closed | No | Step `failed` with an action for the admin ("Add a key for provider X") |
| `internal` | Bug, broken invariant, corrupt row, serialisation failure | **Never** | **Never** | No | Run `failed`, `stop_reason=error`; logged with the span id for support |

A wall-clock timeout on a model call (REQ-LOOP-010) is `transient`, with one special rule: cancel the call and retry **once, on the fallback tier**, instead of three times on the same tier.

### 2. Representation in code

`AppError` gains one structured variant. Existing variants map onto the classes, so current callers keep compiling:

```rust
pub enum ErrorClass { Transient, Policy, Validation, Configuration, Internal }

AppError::Classified {
    class: ErrorClass,
    code: &'static str,          // stable, dotted: "provider.timeout", "policy.denied", ...
    message: String,             // safe to show a user; never contains a secret
    retry_after_ms: Option<u64>, // from Retry-After, when present
    http_status: Option<u16>,
}
```

| Existing variant | Class |
|---|---|
| `NotFound`, `Validation`, `Conflict` | `validation` |
| `PolicyBlocked` | `policy` |
| `Database(busy or locked)` | `transient` |
| `Database(anything else)` | `internal` |

Provider adapters (WP-0146) must return `Classified` with an HTTP-status mapping. Free-text `Validation` for a provider failure is a review blocker.

### 3. Codes

Codes are stable strings, recorded on the span (`lanesra.error.code`), on the step row and in the `run.failed` event:

- **transient:** `provider.timeout`, `provider.rate_limited`, `provider.unavailable`, `provider.network`, `db.busy`
- **policy:** `policy.denied`, `policy.approval_required`, `policy.budget_exhausted`, `policy.data_boundary`, `policy.scope`, `policy.access_denied`
- **validation:** `validation.arguments`, `validation.output_schema`, `validation.guardrail`, `validation.unknown_tool`, `validation.ssrf_blocked`
- **configuration:** `config.missing_key`, `config.provider_disabled`, `config.model_not_found`, `config.not_local`
- **internal:** `internal`

A new code needs only a line in this list. A new class needs a new ADR.

### 4. Gateway failover changes (fixes RAID I-17)

- `transient` (after its retries) and `configuration` move to the next tier.
- `policy` and `validation` stop at once.
- When air-gap is forced, the only allowed tier is a provider marked local. If none is configured, the call fails with `config.not_local`. It never falls back to the workspace default.

### 5. What users see

- The Playground and the run page show the class as a badge (Retrying, Blocked by policy, Needs a fix, Setup needed, Internal error), with the message and the step.
- `internal` errors show a support reference (the span id) and never a stack trace.

## Example

```json
{"type":"run.failed","data":{"step_id":"step_4","class":"configuration","code":"config.not_local",
 "message":"This step handles data classed 'ssn' and must use a local model, but no local provider is configured. Add one under AI Gateway → Local fallback."}}
```

## Consequences

- Retry and failover become predictable and testable. The scripted provider (WP-0005) can return each class, so every row of the table above gets a test.
- Every provider adapter and tool dispatcher must classify its errors. That is mechanical work, spread across WP-0124 and WP-0146.
- Callers that matched on error *text* must switch to matching on `code`.
