---
id: loop/guardrails-section
version: 1
status: active
work_package: WP-0130
variables:
  guardrails_md: { type: text, required: true }
output_contract: "A guardrails section header followed by the guardrails text."
eval_suite: prompts/loop/guardrails-section.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
Operational guardrails (must not be violated):

{{guardrails_md}}
