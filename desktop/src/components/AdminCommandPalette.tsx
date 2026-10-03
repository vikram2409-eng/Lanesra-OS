import { useEffect, useRef, useState } from "react";

import { api } from "../lib/api";
import { sectionFor, type Section } from "./AppShell";
import type { AdminSearchResult, SearchResult } from "../lib/types";

// Admin Control Center Modernization (issue #197): "Command Palette
// (Ctrl/Cmd+K) for navigation/object search/quick create/admin
// destinations, permission-aware." A global modal overlay, not
// `GlobalSearch.tsx`'s always-visible topbar box - that one's own doc
// comment explains why it deliberately isn't a ⌘K overlay (a second,
// desktop-only interaction model for the same record search); this is a
// genuinely different feature (navigation + admin destinations alongside
// record search), so it earns the overlay GlobalSearch declined.
// "Quick create" is scoped honestly: picking a destination navigates to
// that screen, where its own existing "+ New" is one click away, rather
// than this palette growing a second, parallel create flow per entity
// type.
const STATIC_DESTINATIONS: { label: string; section: Section }[] = [
  { label: "Dashboard", section: "dashboard" },
  { label: "Companies", section: "companies" },
  { label: "Contacts", section: "contacts" },
  { label: "Products", section: "products" },
  { label: "Opportunities", section: "opportunities" },
  { label: "Quotes", section: "quotes" },
  { label: "Orders", section: "orders" },
  { label: "Invoices", section: "invoices" },
  { label: "Contracts", section: "contracts" },
  { label: "Tasks", section: "tasks" },
  { label: "Reports", section: "reports" },
  { label: "Assistant", section: "assistant" },
  { label: "Admin", section: "admin" },
];

export function AdminCommandPalette({
  onNavigate,
  onOpenRecord,
  onOpenAdminTab,
  isAdmin,
}: {
  onNavigate: (section: Section) => void;
  onOpenRecord: (section: Section, id: string) => void;
  onOpenAdminTab: (adminTab: string) => void;
  isAdmin: boolean;
}) {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [records, setRecords] = useState<SearchResult[]>([]);
  const [adminResults, setAdminResults] = useState<AdminSearchResult[]>([]);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setOpen((v) => !v);
      } else if (e.key === "Escape") {
        setOpen(false);
      }
    }
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, []);

  useEffect(() => {
    if (open) {
      const handle = setTimeout(() => inputRef.current?.focus(), 0);
      return () => clearTimeout(handle);
    }
    setQuery("");
    setRecords([]);
    setAdminResults([]);
  }, [open]);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setRecords([]);
      setAdminResults([]);
      return;
    }
    const handle = setTimeout(() => {
      api.globalSearch(trimmed).then(setRecords).catch(() => setRecords([]));
      if (isAdmin) api.adminSearch(trimmed).then(setAdminResults).catch(() => setAdminResults([]));
    }, 200);
    return () => clearTimeout(handle);
  }, [query, isAdmin]);

  if (!open) return null;

  const q = query.trim().toLowerCase();
  const destinations = q ? STATIC_DESTINATIONS.filter((d) => d.label.toLowerCase().includes(q)) : STATIC_DESTINATIONS;
  const nothingFound = q.length > 0 && destinations.length === 0 && adminResults.length === 0 && records.length === 0;

  function pickSection(section: Section) {
    onNavigate(section);
    setOpen(false);
  }
  function pickRecord(r: SearchResult) {
    onOpenRecord(sectionFor(r.entity_type), r.entity_id);
    setOpen(false);
  }
  function pickAdmin(r: AdminSearchResult) {
    onOpenAdminTab(r.admin_tab);
    setOpen(false);
  }

  const rowStyle: React.CSSProperties = { display: "block", width: "100%", textAlign: "left", padding: "6px 8px" };
  const groupLabelStyle: React.CSSProperties = { fontSize: 11, color: "var(--text-muted)", padding: "4px 8px", textTransform: "uppercase" };

  return (
    <div
      style={{ position: "fixed", inset: 0, background: "rgba(0,0,0,0.4)", zIndex: 1000, display: "flex", alignItems: "flex-start", justifyContent: "center", paddingTop: "10vh" }}
      onClick={() => setOpen(false)}
    >
      <div className="card" style={{ width: 560, maxWidth: "90vw", maxHeight: "70vh", overflowY: "auto", padding: 0 }} onClick={(e) => e.stopPropagation()}>
        <input
          ref={inputRef}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search or jump to..."
          aria-label="Command palette"
          style={{ width: "100%", border: "none", borderBottom: "1px solid var(--border, #eee)", padding: 12, fontSize: 15, borderRadius: 0 }}
        />
        <div style={{ padding: "4px 0" }}>
          {destinations.length > 0 && (
            <div style={{ marginBottom: 8 }}>
              <div style={groupLabelStyle}>Go to</div>
              {destinations.map((d) => (
                <button key={d.section} type="button" className="link-button" style={rowStyle} onClick={() => pickSection(d.section)}>
                  {d.label}
                </button>
              ))}
            </div>
          )}
          {isAdmin && adminResults.length > 0 && (
            <div style={{ marginBottom: 8 }}>
              <div style={groupLabelStyle}>Admin</div>
              {adminResults.map((r) => (
                <button key={`${r.category}:${r.entity_id}`} type="button" className="link-button" style={rowStyle} onClick={() => pickAdmin(r)}>
                  <div style={{ display: "flex", justifyContent: "space-between", gap: 8 }}>
                    <span style={{ fontSize: 13 }}>{r.title}</span>
                    <span className="badge">{r.category}</span>
                  </div>
                  {r.subtitle && <div style={{ fontSize: 11, color: "var(--text-muted)" }}>{r.subtitle}</div>}
                </button>
              ))}
            </div>
          )}
          {records.length > 0 && (
            <div>
              <div style={groupLabelStyle}>Records</div>
              {records.map((r) => (
                <button key={`${r.entity_type}:${r.entity_id}`} type="button" className="link-button" style={rowStyle} onClick={() => pickRecord(r)}>
                  <div style={{ display: "flex", justifyContent: "space-between", gap: 8 }}>
                    <span style={{ fontSize: 13 }}>{r.title}</span>
                    <span className="badge">{r.entity_type}</span>
                  </div>
                  {r.subtitle && <div style={{ fontSize: 11, color: "var(--text-muted)" }}>{r.subtitle}</div>}
                </button>
              ))}
            </div>
          )}
          {nothingFound && <p className="empty-state">No matches for "{query.trim()}".</p>}
        </div>
      </div>
    </div>
  );
}
