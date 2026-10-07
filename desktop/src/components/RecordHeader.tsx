import type { ReactNode } from "react";

export interface RecordHeaderAttribute {
  label: string;
  value: ReactNode;
}

/**
 * Runtime UX Modernization (issue #198): the one Record Header every
 * object's detail/edit view builds from now on - title/subtitle/status/
 * owner/record number/avatar plus up to 4 key attributes - replacing the
 * 3+ divergent ad hoc headers each entity used to hand-roll (and, for
 * Opportunities and custom objects, headers that didn't exist at all).
 *
 * Also the slot content for `PageRenderer`'s `recordHeader`/`statusBadge`/
 * `owner`/`recordNumber` node types (see that component's own doc
 * comment) - a published Page places those 4 pieces independently in its
 * own grid, so callers feeding PageRenderer pass just the title/subtitle/
 * attributes portion here and the status/owner/number pieces as PageRenderer's
 * other 3 slots directly (StatusBadge/OwnershipByline/plain text) rather
 * than nesting a second RecordHeader inside those slots.
 */
export function RecordHeader({
  avatar,
  title,
  subtitle,
  status,
  owner,
  recordNumber,
  attributes,
}: {
  avatar?: ReactNode;
  title: string;
  subtitle?: string;
  status?: ReactNode;
  owner?: ReactNode;
  recordNumber?: ReactNode;
  attributes?: RecordHeaderAttribute[];
}) {
  const shown = (attributes ?? []).slice(0, 4);
  return (
    <div className="record-header">
      <div className="record-header-top">
        {avatar && <div className="record-header-avatar">{avatar}</div>}
        <div style={{ flex: 1, minWidth: 0 }}>
          <h2 style={{ margin: 0, display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}>
            {title}
            {status}
          </h2>
          {(recordNumber || subtitle) && (
            <p className="muted" style={{ margin: "2px 0 0" }}>
              {recordNumber}
              {recordNumber && subtitle ? " · " : ""}
              {subtitle}
            </p>
          )}
        </div>
      </div>
      {(owner || shown.length > 0) && (
        <div className="record-header-attributes">
          {owner}
          {shown.map((a, i) => (
            <div key={i} className="record-header-attribute">
              <span className="record-header-attribute-label">{a.label}</span>
              <span className="record-header-attribute-value">{a.value}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
