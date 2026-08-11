/**
 * Surfaces `show_message` business rule actions (ADM-BR) that fired on a
 * save - non-blocking by design, so a plain alert is enough: the save
 * already succeeded, this is purely informational. A nicer inline banner
 * is a reasonable next polish item; this keeps every save screen's
 * integration to one line in the meantime.
 */
export function showRuleMessages(messages: string[]): void {
  if (messages.length > 0) {
    alert(messages.join("\n"));
  }
}
