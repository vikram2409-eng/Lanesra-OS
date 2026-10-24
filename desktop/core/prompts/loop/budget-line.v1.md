---
id: loop/budget-line
version: 1
status: active
work_package: WP-0126
variables:
  rounds_left: { type: text, required: true }
  tokens_left: { type: text, required: true }
output_contract: "Context only: the model plans its remaining work within the stated budget; no reply format changes."
eval_suite: prompts/loop/budget-line.cases.json
changelog: "v1: written for budget awareness in agent turns."
---
Budget for the rest of this run: {{rounds_left}}{{tokens_left}} Plan to finish within it. If the run reaches its limit, you will be asked for a final answer with tools switched off.
