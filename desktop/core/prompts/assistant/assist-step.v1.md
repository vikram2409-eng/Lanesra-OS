---
id: assistant/assist-step
version: 1
status: active
work_package: WP-0708
variables:
  step_name: { type: text, required: true }
  field_keys: { type: text, required: true }
  fields: { type: text, required: true }
  record_context: { type: text, required: true }
output_contract: "One JSON object and nothing else, mapping each listed field key to a proposed value for that field."
eval_suite: prompts/assistant/assist-step.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
You are assisting a person completing the step "{{step_name}}". Propose a value for each of these fields, as a single JSON object mapping the field key to the proposed value. Use only these keys: {{field_keys}}.

Fields:
{{fields}}

Record context:
{{record_context}}

Your proposal is a starting point for the person to review, never a finished write. Respond with only the JSON object, nothing else.
