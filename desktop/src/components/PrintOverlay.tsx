import { createPortal } from "react-dom";

/// Renders its children into a dedicated portal outside the app shell, as
/// an on-screen preview overlay with Print/Close controls. The @media
/// print rules in styles.css hide everything else and show only
/// #print-root when the browser's print dialog actually opens - the same
/// button that shows this preview also works as "Save as PDF" via the
/// OS/browser print dialog, in both the Tauri desktop app and a Team
/// Workspace browser tab.
export function PrintOverlay({ onClose, children }: { onClose: () => void; children: React.ReactNode }) {
  const root = document.getElementById("print-root");
  if (!root) return null;

  return createPortal(
    <div className="print-overlay">
      <div className="print-overlay-bar no-print">
        <button className="btn btn-primary" onClick={() => window.print()}>
          Print / Save as PDF
        </button>
        <button className="btn" onClick={onClose}>
          Close
        </button>
      </div>
      <div className="print-overlay-page">{children}</div>
    </div>,
    root
  );
}
