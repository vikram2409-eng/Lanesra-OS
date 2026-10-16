# ADR-0011: Platform prompts live in a versioned catalog of files

- Status: Accepted (program owner, 2026-10-16, PR #628)
- Date: 2026-10-16
- Work package: WP-0009
- Requirements: REQ-PRE-007, REQ-LOOP-016, REQ-CLEAN-001..005 (clean room)
- Constrains: WP-0130 (writes the catalog), WP-0005 (scripted provider), WP-0120, WP-0122, WP-0125, WP-0132, every Phase 3 mode (manager, round table, debate), the online demo

## Context

There are two kinds of prompt:

- **Platform prompts** are written by Lanesra. Today they are Rust string literals spread through the services:
  - the eval judge (`ai_eval_service.rs:129`);
  - the records and admin assistants (`chat_service.rs:885,894`);
  - the section wrappers that `agent_system_prompt` adds around a persona (`chat_service.rs:1458-1491`);
  - the peer-review "APPROVED" instruction (`ai_orchestration_service.rs:289`);
  - the voice planner.

  A change to one is invisible in review unless someone reads the diff closely. Nothing ties a prompt to an evaluation, and nothing records which wording a run used.
- **Workspace prompts** are written by customers: an agent's `system_prompt`, `guardrails_md` and Skill `instructions_md`. They are data, versioned with the agent (`ai_agent_versions`).

The spec requires a prompt catalog written fresh in a calm style, versioned, diffable and evaluated on change. The clean-room rules forbid reusing any other framework's prompt text.

## Decision

### 1. Scope

The catalog holds **platform prompts only**. Workspace prompts stay data and follow the agent version lifecycle (WP-0170). A platform prompt may *wrap* workspace text, for example by adding a heading and the output contract. It never rewrites that text.

### 2. Files

Each prompt version is one Markdown file:

```
desktop/core/prompts/<area>/<name>.v<N>.md
```

Areas include `loop`, `guardrail`, `eval`, `assistant`, `mode` and `voice`. Every file starts with YAML front matter:

```yaml
---
id: eval/judge
version: 2
status: active            # draft | active | retired
work_package: WP-0130
variables:
  task:      { type: text, required: true }
  criteria:  { type: text, required: true }
  response:  { type: text, required: true, untrusted: true }
output_contract: "First line PASS or FAIL; second line one sentence of reasoning."
eval_suite: prompts/eval/judge.cases.json
changelog: "v2: states the output contract first; calmer wording."
---
```

The body follows the front matter. It is plain text with `{{variable}}` placeholders and nothing else: no conditionals, loops or includes. Any logic stays in Rust, where it can be tested.

### 3. Runtime

- **Loading:** files are compiled into the binary (`include_str!`, through a generated registry), so desktop works offline and the files cannot drift from the code.
- **Lookup:** code asks for a prompt by id and version (`prompts::get("eval/judge", 2)`), or by id for the active version.
- **Rendering is strict.** A missing required variable is an error, and so is an unknown placeholder. Variables marked `untrusted: true` are fenced as untrusted content (WP-0145) before they are inserted.
- **Recording:** every model call records the prompt ids and versions it used on its span (`lanesra.prompt.ids`, ADR-0008). Replay and comparison can then show exactly which wording ran.

### 4. Changing a prompt

1. Copy `name.vN.md` to `name.v(N+1).md`, edit the copy, and set the old file to `retired` once nothing pins it. Never edit an active version in place. A diff between the two files is the review.
2. The prompt's `eval_suite` must pass on the scripted provider (WP-0005) in CI, and against recorded runs where they exist. A prompt without a suite is a CI failure from WP-0130 on.
3. **Style rules**, which reviewers check:
   - Calm instructions.
   - State the output contract first.
   - No threats, pressure or role-play urgency ("your job depends on it" is banned).
   - No text copied from another framework (REQ-CLEAN).
   - Lanesra vocabulary only: Team, Playbook, Run, Step.
4. CI checks that ids are unique, that versions are contiguous, that each prompt has a suite, and that every placeholder in the body appears in `variables`, and every variable in the body.

### 5. Online demo

The demo's simulated replies are not prompts. When the demo shows "the prompt this step used", it shows the catalog file's text, copied in at build time by a small script (WP-0013).

## Example

`desktop/core/prompts/loop/final-answer.v1.md`. The loop uses it when a step hits its round limit (REQ-LOOP-008):

```markdown
---
id: loop/final-answer
version: 1
status: active
work_package: WP-0121
variables:
  expected_output: { type: text, required: true }
output_contract: "A final answer in the expected output's format. No tool calls."
eval_suite: prompts/loop/final-answer.cases.json
changelog: "v1: first version."
---
You have used the tool rounds available for this step. Tools are now switched off.

Using what you have found so far, give your best final answer. It should match this expected output:

{{expected_output}}

If something is still unknown, say what is missing in one short sentence at the end.
```

## Consequences

- Every prompt change shows up as a reviewable file diff, with an evaluation behind it.
- Moving the existing inline prompts into the catalog is WP-0130's first task. Behaviour stays the same until a new version is evaluated.
- Prompt files are compiled in, so changing one needs a release. A workspace cannot hot-patch a platform prompt; it changes its own agent's prompts instead.
