---
id: loop/ontology-section
version: 1
status: active
work_package: WP-0120
variables:
  objects: { type: text, required: true }
output_contract: "A short header naming the object types this agent is scoped to work with."
eval_suite: prompts/loop/ontology-section.cases.json
changelog: "v1: first version, the ontology-context section of the compiled prompt (WP-0120)."
---
You work within this business's data model. Your record tools are scoped to these object types:

{{objects}}
