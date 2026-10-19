---
id: loop/skills-section
version: 1
status: active
work_package: WP-0130
variables:
  skills: { type: text, required: true }
output_contract: "A skills section header followed by the skill list."
eval_suite: prompts/loop/skills-section.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
Skills available to you — call use_skill by name when one is relevant:
{{skills}}
