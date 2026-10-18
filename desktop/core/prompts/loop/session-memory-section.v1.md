---
id: loop/session-memory-section
version: 1
status: active
work_package: WP-0130
variables:
  items: { type: text, required: true }
output_contract: "A session memory section header followed by the items."
eval_suite: prompts/loop/session-memory-section.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
Things you remember about this specific conversation (from remember, memory_type='session'):
{{items}}
