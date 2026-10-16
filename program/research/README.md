# Research notes (clean room)

The spec requires a clean room (REQ-CLEAN-001..005). This folder holds **analyst** output only:
plain-language behaviour notes and the spec-vs-code gap audit.

Rules:

- Analyst notes describe *behaviour and ideas* in your own words — never code, prompts,
  function signatures, file layouts or identifiers copied from CrewAI, Microsoft Agent Framework,
  LangGraph or AutoGen.
- An agent acting as an **implementer** in a session must not also act as an analyst on those
  projects in the same session, and must not fetch or open their source.
- The spec's own Research appendix already captures what was learned in October 2026; add notes
  here only when a work package needs more.

Expected files: `gap-audit.md` (WP-0008).
