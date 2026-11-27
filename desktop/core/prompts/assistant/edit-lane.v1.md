---
id: assistant/edit-lane
version: 1
status: active
work_package: WP-0723
variables:
  lane_key: { type: text, required: true }
  current_definition: { type: text, required: true }
  description: { type: text, required: true }
output_contract: "One JSON lane definition file and nothing else: format_version plus the same lane, revised per the change description, with status draft and the version the current definition states."
eval_suite: prompts/assistant/edit-lane.cases.json
changelog: "v1: first version, written fresh for the catalog."
---
You revise a Process Lane from a change description, starting from its current definition. The lane stays the same lane: same id, same object, same name unless the change renames it. Your revision is always a starting point for a person to review, never a finished lane.

The lane to revise:
{{lane_key}}

Its current definition:
{{current_definition}}

The change to make:
{{description}}

Reply with one JSON object and nothing else, the revised lane definition file in this shape:

{
  "format_version": "1.0.0",
  "lane": {
    "id": "{{lane_key}}",
    "name": "...",
    "description": "...",
    "object_key": "...",
    "version": 1,
    "status": "draft",
    "run_data": {"fields": [...]},
    "stages": [...]
  }
}

Rules for the revision:

- Keep the lane id exactly {{lane_key}} and the object_key exactly as the current definition states. The version stays as the current definition states and the status is draft, always.
- Change only what the description asks to change. Keep every stage, step, field and branch the description does not touch, exactly as it is.
- A new step the description asks for goes in the stage where the work happens, with a kebab-case id that no other step in that stage uses. A step the description asks to remove is removed, and any branch that pointed at it now points at the step that follows it, or ends the run with a named outcome. A renamed step keeps its id unless the description renames the work itself.
- Use form steps for work a person does, and check steps only where the description states something to verify.
- Use record steps only where the description states a write to the record, and only with the one adopted Action Type a revision can name without guessing: action_type "<object>.set_status" with inputs {"status": "<the status value the description states>"}, where <object> is the lane's object_key. A write that needs any other Action Type is left out of the revision: the person adds it when refining the draft, once that Action Type is adopted. Do not invent action type names.
- Use approval steps only where the description states a decision by a person. An approval step names the role that decides, carries the writes the decision posts when it is approved (the same adopted set_status write as a record step, with the status value the description states for approval, or the decision word itself, written in upper case, when the description states only the decision), and declares an approved branch and a rejected branch. Each branch names a real later step as its target, or ends the run with a named outcome.
- Use agent steps only where the description names the agent that does the work, as assistance with autonomy propose and agent set to the name the description gives. A form step may carry the same assistance where the description says an agent proposes something for a person to confirm. Never pick an agent the description does not name.
- Every step except a check step names the role that does it. A check step names no role.
- A form step names the role that does it, carries a short description, and lists the fields the person fills in. Each field binds to a run-data field declared under run_data.fields, as run_data.<key>, or to a record field, as record.<field>. Declare every run-data field the steps use, once, under run_data.fields, with a type of string, integer, number, boolean or date.
- A check step names no role and carries a check rule with scope anchor, a message, a condition and a pass branch and a fail branch. A condition compares one field to a value or to another field, using a Business Rules operator such as equals. It may only name a run-data field declared under run_data.fields, as $run_data.<key>, or $record.status. Each branch names a real later step as its target, or ends the run with a named outcome.
- Stage and step ids are kebab-case and unique within their stage. Step names are plain words, never ids.
- Do not invent record fields, action types, agents or record values. If the description does not name something, leave it out of the revision rather than guessing it.
- Keep the revision small: the change the description actually states, and no more.
