import { useEffect, useRef, useState } from "react";

import { api } from "../lib/api";
import { sectionFor, type Section } from "./AppShell";
import type { CustomObjectDefinition, SearchResult } from "../lib/types";

// `sectionFor` (entity_type -> nav Section, == SearchResult.entity_type)
// now lives in AppShell.tsx, alongside Section/customObjectSection - see
// that module's own doc comment for why. Re-exported here so anything
// that already imports it from this module (Dashboard.tsx) keeps working.
export { sectionFor };

const BUILT_IN_ENTITY_TYPES = ["Company", "Contact", "Opportunity", "Quote", "Order", "Invoice", "Contract", "Task", "Product"];

function groupLabel(entityType: string, customObjects: CustomObjectDefinition[]): string {
  if (BUILT_IN_ENTITY_TYPES.includes(entityType)) return entityType;
  return customObjects.find((o) => o.key === entityType)?.plural_label ?? entityType;
}

/**
 * Global search (roadmap "Global search & list-view filtering"): a
 * command-palette-style box in the topbar. Deliberately not a modal/⌘K
 * overlay - a always-visible box matches the online demo's own search
 * input rather than introducing a second, desktop-only interaction model
 * for the same feature (see app.js's runSearch).
 */
export function GlobalSearch({
  customObjects,
  onOpenResult,
}: {
  customObjects: CustomObjectDefinition[];
  onOpenResult: (section: Section, id: string) => void;
}) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const trimmed = query.trim();
    if (trimmed.length < 2) {
      setResults([]);
      setLoading(false);
      return;
    }
    setLoading(true);
    const handle = setTimeout(() => {
      api
        .globalSearch(trimmed)
        .then((r) => setResults(r))
        .catch(() => setResults([]))
        .finally(() => setLoading(false));
    }, 250);
    return () => clearTimeout(handle);
  }, [query]);

  useEffect(() => {
    function onClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener("mousedown", onClickOutside);
    return () => document.removeEventListener("mousedown", onClickOutside);
  }, []);

  function pick(result: SearchResult) {
    onOpenResult(sectionFor(result.entity_type), result.entity_id);
    setQuery("");
    setResults([]);
    setOpen(false);
  }

  const showDropdown = open && query.trim().length >= 2;

  return (
    <div ref={containerRef} style={{ position: "relative", width: 320 }}>
      <input
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          setOpen(true);
        }}
        onFocus={() => setOpen(true)}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            setOpen(false);
            (e.target as HTMLInputElement).blur();
          }
        }}
        placeholder="Search companies, contacts, quotes..."
        aria-label="Global search"
        style={{ width: "100%" }}
      />
      {showDropdown && (
        <div
          className="card"
          style={{
            position: "absolute", left: 0, top: "calc(100% + 4px)", width: "100%", maxHeight: 420, overflowY: "auto",
            zIndex: 20, boxShadow: "0 4px 16px rgba(0,0,0,0.15)",
          }}
        >
          {loading && <p className="empty-state">Searching...</p>}
          {!loading && results.length === 0 && <p className="empty-state">No matches for "{query.trim()}".</p>}
          {!loading &&
            results.map((r) => (
              <button
                key={`${r.entity_type}:${r.entity_id}`}
                type="button"
                className="link-button"
                onClick={() => pick(r)}
                style={{
                  display: "block", width: "100%", textAlign: "left", padding: "6px 4px",
                  borderBottom: "1px solid var(--border, #eee)",
                }}
              >
                <div style={{ display: "flex", justifyContent: "space-between", gap: 8 }}>
                  <span style={{ fontSize: 13 }}>{r.title}</span>
                  <span className="badge" style={{ flexShrink: 0 }}>{groupLabel(r.entity_type, customObjects)}</span>
                </div>
                {r.subtitle && (
                  <div style={{ fontSize: 11, color: "var(--text-muted)" }}>{r.subtitle}</div>
                )}
              </button>
            ))}
        </div>
      )}
    </div>
  );
}
