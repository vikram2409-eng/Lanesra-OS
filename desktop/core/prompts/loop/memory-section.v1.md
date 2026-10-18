---
id: loop/memory-section
version: 1
status: active
work_package: WP-0130
variables:
  memory_md: { type: text, required: true }
output_contract: "A memory section header followed by the memory text."
eval_suite: prompts/loop/memory-section.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
Your persistent memory from prior runs (revise it with update_memory whenever something worth remembering happens):

{{memory_md}}
