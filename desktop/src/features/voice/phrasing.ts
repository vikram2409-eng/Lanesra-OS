import type { VoicePlanStep } from "../../lib/types";

/**
 * Voice-First Mode, PR 3 (spec §16): turns a plan step's already-planned
 * `description`/`brief_description`/`detail_note` into the text that's
 * actually spoken, per the user's `spoken_detail` preference - the one
 * seam `speak()` and its app.js `voiceSpeak` mirror both call through, so
 * the setting affects real output instead of sitting unread. Never
 * fabricates new content: "short" and "detailed" both only ever say what
 * the planner already knew when it built the step.
 */
export function spokenTextForStep(step: VoicePlanStep, detail: "short" | "normal" | "detailed"): string {
  if (detail === "short") return step.brief_description || step.description;
  if (detail === "detailed") return step.detail_note ? `${step.description}. ${step.detail_note}` : step.description;
  return step.description;
}

export function spokenTextForSteps(steps: VoicePlanStep[], detail: "short" | "normal" | "detailed"): string {
  return steps.map((s) => spokenTextForStep(s, detail)).join(". ");
}

/**
 * `notes` (a RUN_AGENT/RUN_PIPELINE step's real reply, spoken after
 * confirmation - see `VoiceExecutionResult.notes`) is free-form text the
 * planner didn't shape into brief/detailed variants, so "short" is the one
 * tier this can honestly do anything about: speak only the first note
 * rather than every one, instead of inventing a summary of text we didn't
 * write.
 */
export function spokenTextForNotes(notes: string[], detail: "short" | "normal" | "detailed"): string {
  if (detail === "short") return notes.slice(0, 1).join(". ");
  return notes.join(". ");
}
