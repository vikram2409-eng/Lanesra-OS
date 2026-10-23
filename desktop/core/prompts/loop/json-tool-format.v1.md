---
id: loop/json-tool-format
version: 1
status: active
work_package: WP-0122
variables:
  tools: { type: text, required: true }
output_contract: "One JSON object and nothing else: {\"answer\": \"...\"} or {\"tool_calls\": [{\"name\": \"...\", \"arguments\": {...}}]}."
eval_suite: prompts/loop/json-tool-format.cases.json
changelog: "v1: first version, for models without native tool calling."
---
Answer with one JSON object and nothing else: no Markdown fences and no text around it.

To give your answer, send:
{"answer": "<your answer>"}

To use tools, send:
{"tool_calls": [{"name": "<tool name>", "arguments": {<the tool's parameters>}}]}

The tools you can use, with their parameters as JSON Schema:
{{tools}}

Use only these tools, and give each call a JSON object of arguments that matches its parameters. If a call comes back with an error, read it, correct the call and send it again.
