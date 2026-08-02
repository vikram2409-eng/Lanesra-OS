const POSITIVE = new Set(["Won", "Accepted", "Fulfilled", "Paid", "Active", "Active Customer", "Confirmed"]);
const NEGATIVE = new Set(["Lost", "Rejected", "Cancelled", "Overdue", "Void", "Expired"]);
const NEUTRAL_WARNING = new Set(["Partially Paid", "Partially Fulfilled", "Expiring", "Sent", "Viewed"]);

export function StatusBadge({ status }: { status: string }) {
  let className = "badge";
  if (POSITIVE.has(status)) className += " badge-success";
  else if (NEGATIVE.has(status)) className += " badge-danger";
  else if (NEUTRAL_WARNING.has(status)) className += " badge-warning";
  return <span className={className}>{status}</span>;
}
