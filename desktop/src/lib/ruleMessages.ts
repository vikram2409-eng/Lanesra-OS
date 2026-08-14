import type { SaveNotices } from "./types";

/**
 * Surfaces the non-blocking show_error/show_warning business rule notices
 * (ADM-BR, second addendum round) that fired on a save - non-blocking by
 * design, so a plain alert is enough: the save already succeeded, this is
 * purely informational. Errors are prefixed to read more urgently than
 * warnings even though neither ever blocked the save (that's block_save's
 * job). A nicer inline banner is a reasonable next polish item; this keeps
 * every save screen's integration to one line in the meantime.
 */
export function showRuleMessages(notices: SaveNotices): void {
  const lines = [...notices.errors.map((m) => `⚠ ${m}`), ...notices.warnings];
  if (lines.length > 0) {
    alert(lines.join("\n"));
  }
}
