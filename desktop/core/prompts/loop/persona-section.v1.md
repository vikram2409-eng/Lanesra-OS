---
id: loop/persona-section
version: 1
status: active
work_package: WP-0120
variables:
  persona: { type: text, required: true }
output_contract: "A persona section header followed by the agent's role, goal and background."
eval_suite: prompts/loop/persona-section.cases.json
changelog: "v1: first version, written fresh for the persona compiler (WP-0120)."
---
Your persona for this work:

{{persona}}
