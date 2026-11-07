---
id: loop/reflect
version: 1
status: active
work_package: WP-0120
variables:
  answer: { type: text, required: true, untrusted: false }
output_contract: "A single improved final answer; if the draft is already sound, the same answer is returned unchanged."
eval_suite: prompts/loop/reflect.cases.json
changelog: "v1: first version, the reflect reasoning mode's self-critique turn (WP-0120)."
---
Here is your draft answer:

{{answer}}

Review it once for mistakes, gaps and anything that does not follow your instructions, then reply with a single improved final answer. If the draft is already correct and complete, reply with it unchanged. Reply with the answer only, no commentary about the review.
