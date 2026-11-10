---
id: loop/plan-first
version: 1
status: active
work_package: WP-0120
variables: {}
output_contract: "A short numbered plan of the steps the agent intends to take, as plain text; no tool calls."
eval_suite: prompts/loop/plan-first.cases.json
changelog: "v1: first version, the plan-first reasoning mode's planning turn (WP-0120)."
---
Before taking any action, write a short numbered plan of the steps you intend to take to finish this task. List the tools you expect to use. Do not call any tool yet; reply with the plan only.
