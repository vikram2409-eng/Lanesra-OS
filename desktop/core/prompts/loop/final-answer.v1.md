---
id: loop/final-answer
version: 1
status: active
work_package: WP-0121
variables:
  reason: { type: text, required: true }
output_contract: "A plain-text answer built only from what the run has already found; no tool calls."
eval_suite: prompts/loop/final-answer.cases.json
changelog: "v1: written for step limits and partial results."
---
This run has reached its limit: {{reason}}. Tools are switched off for this last reply.

Write your final answer now from what you have already found. Say plainly what is done, what is not, and what a person would need to do to finish it. Do not ask to call another tool.
