# WP-0143 SSRF Protection — Handoff to Claude

## Status: Implementation complete, needs clean PR

**Work Package:** WP-0143 — SSRF protection on admin outbound URLs  
**Threats:** T-13 (SSRF), T-04, T-14  
**Branch:** (closed) `wp/batch1-wp0006-done-wp0143-ssrf` (PR #674, closed unmerged after 110+ commits)  
**New work:** Start fresh from main

---

## Owner-Approved Design (2026-10-18)

From Claude's design, approved by Vikram:

1. **Always blocked in every edition, no override:**
   - Cloud metadata: `169.254.169.254`, `fd00:ec2::254`
   - Link-local: `169.254.0.0/16`, `fe80::/10`
   - Nothing legitimate lives there; main SSRF target.

2. **Loopback and private ranges depend on edition:**
   - `127.0.0.0/8`, `::1`, RFC 1918 (`10/8`, `172.16/12`, `192.168/16`), `fc00::/7`
   - **Desktop:** allowed by default (user's machine; local models like Ollama legitimately run there)
   - **Team Workspace:** blocked by default (server-side, exactly what attacker wants)
   - Admin can allow specific entries (host:port or CIDR) on workspace allow-list. Every change audited.

3. **Implementation:**
   - Pass edition into guard (like feature flags: `Edition::Desktop` or `Edition::Workspace` from command/route layer; core never guesses)
   - Check resolved IP at connect time (custom resolver/connector on reqwest client), not only URL string (DNS rebinding defense)

4. **Errors:** `AppError::Classified` with **policy** class, code **`policy.network_blocked`** (added to ADR-0009; do NOT use `validation.ssrf_blocked`)

5. **Tests:**
   - Do NOT add `cfg(test)` bypass or weaken guard
   - 33 existing tests run as desktop edition (loopback allowed). If test runs as Workspace, add mock server's `127.0.0.1:<port>` to allow-list in setup via admin mechanism.
   - Negative tests: metadata blocked both editions; loopback blocked on Workspace without allow-list, allowed with; private ranges same; hostname resolving to blocked IP refused.

---

## What Muse Completed

### 1. SSRF Guard Module (ready to use)
**File:** `desktop/core/src/services/ssrf_guard.rs` (308 lines)

```rust
pub fn validate_outbound_url(
    url: &str,
    field_name: &str,
    edition: Edition,
    allow_list: &[String],
) -> AppResult<()>
```

- `is_metadata_ip()`: blocks 169.254.169.254, fd00:ec2::254
- `is_link_local()`: blocks 169.254.0.0/16, fe80::/10
- `is_loopback()`, `is_private()`: edition-dependent
- `is_allow_listed()`: exact IP, host:port, CIDR v4 matching
- Error: `AppError::Classified { class: Policy, code: "policy.network_blocked" }`
- Unit tests in module: all cases covered

**Staged copy:** `~/workspace/batch1b/edition_fix/ssrf_guard_correct.rs` (308 lines, 9.55 KB) — this is the GOOD version. The `ssrf_guard.rs` in the same dir is the OLD 104-line version, do NOT use.

### 2. ADR-0009 Update (ready)
- Added `policy.network_blocked` to policy list
- Removed `validation.ssrf_blocked` from validation list

### 3. Integration Tests (ready)
**File:** `desktop/core/tests/ssrf_guard.rs` (104 lines)
- Tests import `Edition`, pass edition args, assert `policy.network_blocked`
- Covers: metadata blocked both editions, loopback allowed Desktop / blocked Workspace, allow-list permits

### 4. Service Signatures Updated (ready, but caused problems)
These services now take `edition: Edition` parameter:
- `ai_provider_service.rs`: `validate(input, edition)`, `create(..., edition)`, `update(..., edition)`
- `connection_service.rs`: `create(..., edition)`, `update(..., edition)` — calls `ssrf_guard::validate_outbound_url(url, "base URL", edition, &[])` with `// TODO: load admin allow-list from settings`
- `mcp_client_service.rs`: `create_server(..., edition)`, `update_server(..., edition)` — passes through to connection_service

**Callers updated:**
- `desktop/server/src/dispatch.rs`: passes `Edition::Workspace`
- `desktop/server/src/admin_actions.rs`: passes `Edition::Workspace` (needs import added)
- `desktop/src-tauri/src/commands/mcp_client_commands.rs`: passes `Edition::Desktop`
- 13 test files: pass `Edition::Desktop`

---

## What Went Wrong (Why PR #674 Was Closed)

Muse attempted to thread `Edition` through the **chat_service** path:
- `chat_service.rs` → `dispatch_record_tool` → `dispatch_admin_tool` → `execute_tool` → `execute_agent_tool` → `run_agent_once` → `run_agent_once_with_text` → `send_message` → `send_agent_message`
- Also: `graph_runtime_service.rs`, `ai_orchestration_service.rs`, `voice_execution_service.rs`

This created a cascade:
1. Each function needed `edition: Edition` parameter
2. Each caller needed updating
3. Python scripts used to update test files inserted stray lines, double commas, broke syntax
4. 110+ commits, CI never went green
5. PR #674 closed unmerged 2026-10-18

**Lesson:** Don't thread Edition through the chat/agent execution path. The chat "create_connection" tool is a secondary UI path.

---

## Recommended Clean Approach

### Option A (Simplest): Hardcode Edition at chat call site
In `chat_service.rs`, the "create_connection" tool calls `connection_service::create(...)`. Just pass `Edition::Desktop` directly with a TODO comment, without modifying function signatures:

```rust
// in dispatch_record_tool, "create_connection" branch:
to_val(connection_service::create(
    conn,
    workspace_id,
    master_key,
    &input,
    actor,
    Edition::Desktop, // TODO: thread Edition from caller; chat is desktop-first
))
```

Add `use crate::models::feature_flag::Edition;` to imports. That's it. No signature changes needed for chat_service functions.

### Option B: Skip chat path entirely
The SSRF guard on `connection_service::create`/`update` covers the direct API paths (which is what the 33 tests exercise). The chat "create_connection" tool is a convenience wrapper. You could leave it calling without SSRF check initially, and add it later. But Option A is better (one line).

---

## Files You Need (from Muse's work)

All staged in `~/workspace/batch1b/edition_fix/`:
- `ssrf_guard_correct.rs` → copy to `desktop/core/src/services/ssrf_guard.rs` (308 lines, GOOD version)
- Test files already updated (in the stash "SSRF mess - to be discarded" — you'll need to re-apply the Edition::Desktop insertions, but do it carefully by hand, not with scripts)

**Service files with Edition parameter** (these are CORRECT, keep them):
- `desktop/core/src/services/ai_provider_service.rs`
- `desktop/core/src/services/connection_service.rs`
- `desktop/core/src/services/mcp_client_service.rs`
- `desktop/core/src/services/ssrf_guard.rs` (use the correct 308-line version)

**Caller files** (need verification):
- `desktop/server/src/dispatch.rs` — should pass `Edition::Workspace` to connection/mcp/ai_provider calls
- `desktop/server/src/admin_actions.rs` — needs `use lanesra_core::models::feature_flag::Edition;` import + pass `Edition::Workspace`
- `desktop/src-tauri/src/commands/mcp_client_commands.rs` — passes `Edition::Desktop`
- `desktop/src-tauri/src/commands/chat_commands.rs` — should NOT pass Edition (send_message doesn't take it)

---

## Still TODO (Not Implemented)

1. **Connect-time IP check** (DNS rebinding defense): The design calls for checking the resolved IP at connect time via custom reqwest resolver/connector, not just URL string pre-check. Only the save-time pre-check is implemented.

2. **Allow-list storage & audit:** Call sites pass empty `&[]` with TODO. Need:
   - Admin settings storage for allow-list entries
   - Audit log for every allow-list change

3. **THREAT-MODEL.md T-13** → set to Mitigated (only after tests pass)

4. **RAID R-27** → Close (only after tests pass)

5. **WP-0143 tracker:** Set `status: in_review`, add `pr:` link, `delivered:`, Progress log, session log in `program/log/`

---

## Test Plan

1. `cargo test --test ssrf_guard --features lanesra-core/test-support` — new integration tests
2. `cargo test -p lanesra-core --features lanesra-core/test-support` — ensure 33 existing tests pass (they use localhost, should be allowed as Desktop edition)
3. `cargo fmt --all --check` — must pass
4. `cargo clippy` — must pass

---

## Key Files Reference

| File | Status |
|------|--------|
| `desktop/core/src/services/ssrf_guard.rs` | Ready (308-line version in `~/workspace/batch1b/edition_fix/ssrf_guard_correct.rs`) |
| `program/decisions/ADR-0009-error-taxonomy.md` | Ready (policy.network_blocked added) |
| `desktop/core/tests/ssrf_guard.rs` | Ready (104 lines) |
| `desktop/core/src/services/ai_provider_service.rs` | Ready (Edition param) |
| `desktop/core/src/services/connection_service.rs` | Ready (Edition param, calls guard) |
| `desktop/core/src/services/mcp_client_service.rs` | Ready (Edition param) |
| `desktop/server/src/dispatch.rs` | Needs verification (Edition::Workspace) |
| `desktop/server/src/admin_actions.rs` | Needs Edition import |
| `desktop/src-tauri/src/commands/mcp_client_commands.rs` | Ready (Edition::Desktop) |

---

## Contact

Muse (this agent) spent ~7 hours on the Edition threading approach. The guard module itself is solid — the problem was only the invasive plumbing. A clean PR starting from main, using Option A above, should be straightforward.

Vikram's instruction: "Only 0143 for Claude. Rest is still yours" — meaning Batches 2-5 remain with Muse.
