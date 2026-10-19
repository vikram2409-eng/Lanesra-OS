# ADR-0017: Rust core; Python for the SDK, evaluations and a sandboxed tool runtime

- Status: Accepted (decided by the program owner, Vikram Grover, 2026-10-19)
- Follows: ADR-0013 §4 and ADR-0014 (several workspaces per server, SaaS later, PostgreSQL option)
- Editions: Desktop, Team Workspace, online demo; a future Cloud SaaS edition

## Context

The owner asked whether the Agent Playground should be ported to Python, and whether that would
help a future SaaS edition scale. Measured on 2026-10-19:

- About 136,000 lines of Rust: about 70,000 in the core engine (`desktop/core/src`), 51,000 of tests,
  7,000 in the Tauri desktop shell and 8,000 in the Team Workspace server.
- About 50,000 lines of React/TypeScript screens, shared by desktop, Team Workspace and the demo.
- Python is used only for build and program tooling (`program/programctl.py`, repository scripts). None
  of it ships in the product.

## Decision

1. **The product core stays in Rust.** Desktop (Tauri) and the Team Workspace server keep one shared
   engine, so a feature is written once and ships in every edition in the same release (ADR-0013 §3).
2. **Python is used at the edges, where its ecosystem helps:**
   - **Python SDK** for the public API, so customers can script agents and integrations.
   - **Evaluations:** the evaluation harness and offline analysis may be written in Python. They run
     outside the product and call it through the public API or the CLI.
   - **Sandboxed Python tool runtime:** agents may run user-written Python tools in an isolated
     process the Rust core starts and controls. Tool calls go through the same policy, approval and
     audit path as every other tool. The sandbox has no access to the workspace database, data key or
     credentials, and network access follows the WP-0143 rules.
3. **No port of existing Rust code to Python.** A proposal to move a part of the core needs a new ADR.

## Why

- A port would take several months with no new features, and would re-open work that is already done
  and tested.
- Tauri needs a Rust backend. A Python desktop app means bundling a Python runtime: bigger installers,
  harder signing and updates, slower start-up.
- Moving only the server would mean two codebases and every feature written twice.
- SaaS scalability comes from the architecture (servers that can be copied side by side, PostgreSQL,
  a job queue for agent runs, per-workspace isolation in ADR-0014), not from the language. Rust uses
  less memory and CPU per user, which makes hosting cheaper.
- Rust's memory safety and type checks suit a product that holds credentials and makes outbound
  calls on users' behalf.

## Consequences

- Work packages for the SDK, the evaluation harness and the Python tool runtime are to be added
  when they are planned; the tool runtime needs its own threat-model entry before it is built.
- AI agents writing Rust make small changes and let CI compile them; no bulk scripted edits.
- Revisit only if the product's core needs change (for example, a Python-only library the core must
  call in-process).
