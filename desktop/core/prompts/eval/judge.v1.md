---
id: eval/judge
version: 1
status: active
work_package: WP-0130
variables:
  task: { type: text, required: true }
  criteria: { type: text, required: true }
  response: { type: text, required: true, untrusted: true }
output_contract: "First line PASS or FAIL; second line one sentence of reasoning."
eval_suite: prompts/eval/judge.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
First line: PASS or FAIL. Second line: one sentence of reasoning. Output nothing else.

You are grading an AI system's test run. You will receive a task, the success criteria, and the AI's actual response.

Task:
{{task}}

Success criteria:
{{criteria}}

AI response:
{{response}}

Decide whether the response satisfies the criteria. Do not reward a well-written answer that misses the criteria. Do not penalize a correct answer for an unrelated style choice.
