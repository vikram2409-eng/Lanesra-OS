# Threat Model — LanesraOS Agent Playground (WP-0011)

**Status:** Draft — pending owner review (WP-0011 AC-1)  
**Date:** 2026-10-17  
**Owner:** muse (WP-0011)

This is the **only** threat list for the program (per WP-0011). All work packages
touching tools, approvals, or auth must link to the threats they mitigate (WP-0011 AC-3).

## Assets

1. **Business records** — companies, contacts, opportunities, and custom objects in SQLite
2. **Agent credentials** — API keys and tokens for model providers and MCP servers
3. **User credentials** — passwords, session tokens
4. **Audit logs** — run history, approval decisions, tool call traces
5. **Source code and configuration** — the LanesraOS codebase itself

## Trust Boundaries

| Boundary | Trusts | Does not trust |
|----------|--------|----------------|
| UI (desktop/web) | Server API | Direct DB access |
| Server | Core library, authenticated users | Raw client input, model provider output |
| Core | SQLite, local filesystem | Network input |
| Model providers | API keys | Prompt content (may be injected) |
| MCP servers | Declared tool schemas | Tool output content |
| Connectors | OAuth tokens | Third-party API responses |

## Threats (STRIDE + LLM-specific)

### T-01: Prompt injection (direct)
**Category:** Tampering / LLM-01  
**Description:** Attacker crafts a prompt that makes the agent ignore its instructions.  
**Mitigations:** Input sanitization (REQ-GOV-XXX); system prompt hardening; output validation.  
**Covered by:** WP-0005 (scripted provider for testing), future governance WPs.

### T-02: Prompt injection (indirect)
**Category:** Tampering / LLM-01  
**Description:** Malicious content in a business record (e.g., email body) injects instructions when the agent reads it.  
**Mitigations:** Content sandboxing; mark untrusted data; approval gates for sensitive actions.  
**Covered by:** Future governance WPs.

### T-03: Excessive agency
**Category:** Elevation of privilege / LLM-08  
**Description:** Agent takes actions beyond its intended scope (e.g., deletes records it should only read).  
**Mitigations:** Least-privilege tool grants; approval workflows; action budgets.  
**Covered by:** WP-0100 (principal model), WP-0111/WP-0112 (budgets).

### T-04: Data exfiltration via tools
**Category:** Information disclosure / LLM-06  
**Description:** Agent sends sensitive business data to an external tool or model provider.  
**Mitigations:** Data classification; egress filtering; audit logging of tool calls.  
**Covered by:** Future governance WPs.

### T-05: Approval spoofing
**Category:** Spoofing  
**Description:** Attacker forges an approval decision to bypass human oversight.  
**Mitigations:** Cryptographic approval tokens; tamper-evident audit log.  
**Covered by:** Future governance WPs.

### T-06: Cost exhaustion
**Category:** Denial of service / LLM-10  
**Description:** Attacker triggers excessive model API calls, incurring cost.  
**Mitigations:** Per-run cost budgets; rate limiting; anomaly detection.  
**Covered by:** WP-0111/WP-0112 (budgets).

### T-07: Memory poisoning
**Category:** Tampering  
**Description:** Attacker plants false information in long-term memory that influences future agent behavior.  
**Mitigations:** Memory provenance tracking; human review of memory writes.  
**Covered by:** Future governance WPs.

### T-08: Dependency vulnerability
**Category:** Tampering  
**Description:** Vulnerable Rust or npm dependency allows code execution.  
**Mitigations:** Automated scanning (cargo-deny, npm audit) in CI; fail on high-severity.  
**Covered by:** WP-0011 (this WP).

### T-09: Secret leakage
**Category:** Information disclosure  
**Description:** API keys or credentials committed to the repo.  
**Mitigations:** Secret scanning (gitleaks) in CI; pre-commit hooks.  
**Covered by:** WP-0011 (this WP).

### T-10: SQL injection
**Category:** Tampering  
**Description:** Malicious input crafted to alter SQL queries.  
**Mitigations:** Parameterized queries (rusqlite); input validation.  
**Covered by:** Existing code patterns; WP-0007 (migration harness).

## Gaps → RAID

Threats without full mitigation become RAID risks. See `program/RAID.md`.
