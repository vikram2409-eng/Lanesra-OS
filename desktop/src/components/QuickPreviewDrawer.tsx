import type { ReactNode } from "react";

/**
 * Runtime UX Modernization (issue #198): a right-side slide-in peek at a
 * record's key fields without leaving the list - `ListTable`'s row click
 * opens this instead of navigating away; the record's own id/number link
 * (rendered by the caller's column `render`) still navigates straight to
 * the full detail page, same as before. Reuses the row object already in
 * memory from the list query - no extra fetch.
 */
export function QuickPreviewDrawer({
  title,
  subtitle,
  fields,
  onClose,
  onOpenFull,
}: {
  title: string;
  subtitle?: string;
  fields: Array<{ label: string; value: ReactNode }>;
  onClose: () => void;
  onOpenFull: () => void;
}) {
  return (
    <div className="quick-preview-overlay" onClick={onClose}>
      <div className="quick-preview-drawer" onClick={(e) => e.stopPropagation()}>
        <div className="quick-preview-header">
          <div>
            <h3 style={{ margin: 0 }}>{title}</h3>
            {subtitle && <p className="muted" style={{ margin: "2px 0 0" }}>{subtitle}</p>}
          </div>
          <button className="btn" onClick={onClose}>
            Close
          </button>
        </div>
        <div className="quick-preview-fields">
          {fields.map((f, i) => (
            <div key={i} className="quick-preview-field">
              <span className="quick-preview-label">{f.label}</span>
              <span className="quick-preview-value">{f.value}</span>
            </div>
          ))}
        </div>
        <div className="quick-preview-footer">
          <button className="btn btn-primary" onClick={onOpenFull}>
            Open full record →
          </button>
        </div>
      </div>
    </div>
  );
}
