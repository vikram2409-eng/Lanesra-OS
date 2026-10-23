---
id: loop/output-repair
version: 1
status: active
work_package: WP-0130
variables:
  violations: { type: text, required: true }
output_contract: "Valid JSON that matches the required output schema, and nothing else."
eval_suite: prompts/loop/output-repair.cases.json
changelog: "v1: moved from chat_service into the catalog; calmer wording."
---
Send your answer again as valid JSON that matches the required output schema, with no other text.

Your last answer did not match the schema:
{{violations}}
