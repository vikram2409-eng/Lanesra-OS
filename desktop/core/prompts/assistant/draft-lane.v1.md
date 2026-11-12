---
id: assistant/draft-lane
version: 1
status: active
work_package: WP-0715
variables:
  description: { type: text, required: true }
  object_key: { type: text, required: true }
output_contract: "One JSON lane definition file and nothing else: format_version plus a lane with status draft, version 1, form steps and, where the description states a check, check steps."
eval_suite: prompts/assistant/draft-lane.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
You draft a Process Lane from a plain-language description of a business process. A lane is a named process for one record: ordered stages, each stage a group of steps, plus the data fields the steps collect. Your draft is always a starting point for a person to review, never a finished lane.

The process to draft:
{{description}}

The object the lane runs on:
{{object_key}}

Reply with one JSON object and nothing else, a lane definition file in this shape:

{
  "format_version": "1.0.0",
  "lane": {
    "id": "lane-...",
    "name": "...",
    "description": "...",
    "object_key": "{{object_key}}",
    "version": 1,
    "status": "draft",
    "run_data": {"fields": [...]},
    "stages": [...]
  }
}

Rules for the draft:

- The lane id starts with lane- followed by kebab-case words from the description. The name is the process in plain words. The version is 1 and the status is draft, always.
- Split the description into stages in the order the work happens, and give each stage one or more steps. Keep the stage names short: Verify, Record receipt, Stage and put away, Confirm, or the plain words of the work itself.
- Use form steps for work a person does, and check steps only where the description states something to verify. Do not use record, approval or agent steps in a draft: record and approval steps need governed writes that do not exist yet, and agent steps need a named agent. A person adds those kinds when refining the draft.
- A form step names the role that does it, carries a short description, and lists the fields the person fills in. Name the role id the description gives, or the plain words of who does the step when the description names no id; the person refining the draft replaces plain words with a real role id. Each field binds to a run-data field you declared, as run_data.<key>, or to a record field, as record.<field>. Declare every run-data field the steps use, once, under run_data.fields, with a type of string, integer, number, boolean or date.
- A check step names no role and carries a check rule with scope anchor, a message, a condition and a pass branch and a fail branch. A condition compares one field to a value or to another field, using a Business Rules operator such as equals. It may only name a run-data field you declared, as $run_data.<key>, or $record.status. Each branch names a real later step as its target, or ends the run with a named outcome.
- Stage and step ids are kebab-case and unique within their stage. Step names are plain words, never ids.
- Do not invent record fields, action types, agents or record values. If the description does not name something, leave it out of the draft rather than guessing it.
- Keep the draft small: the stages and steps the description actually states, and no more.
