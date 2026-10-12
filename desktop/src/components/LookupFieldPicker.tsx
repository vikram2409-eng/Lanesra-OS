import { useEffect, useMemo, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { listRecordsOfType, type PickableRecord } from "../lib/recordPicker";

/**
 * The Lookup Custom Field Type's picker: search-as-you-type against the
 * target entity type's records, with keyboard navigation and a clear
 * affordance - the interaction model `opensourceui`'s combobox-field-input
 * demonstrates, rebuilt in plain React + this codebase's own CSS-variable
 * system (no Tailwind, no new dependencies). Replaces the "eagerly load
 * every record into a plain `<select>`" pattern Contact's `company_id` and
 * Task's `related_id` pickers use - fine at demo scale, but this is the
 * one new picker introduced for the Lookup field type, so it's worth
 * building the better version once rather than repeating the old one.
 *
 * `value` is the linked record's id (or "" for no link); `onChange` is
 * called with the new id, or "" to clear. The underlying list is still
 * fetched in full per `targetEntityType` (see `listRecordsOfType`) and
 * filtered client-side - fine at this app's scale; a server-side paginated
 * search is a reasonable follow-up if a target type's record count ever
 * makes that eager fetch itself the bottleneck, not something this picker
 * needs to solve today.
 */
export function LookupFieldPicker({
  value,
  onChange,
  targetEntityType,
  disabled,
  required,
  placeholder,
}: {
  value: string;
  onChange: (id: string) => void;
  targetEntityType: string;
  disabled?: boolean;
  required?: boolean;
  placeholder?: string;
}) {
  const records = useQuery({
    queryKey: ["pickableRecords", targetEntityType],
    queryFn: () => listRecordsOfType(targetEntityType),
  });
  const byId = useMemo(() => new Map((records.data ?? []).map((r) => [r.id, r])), [records.data]);
  const selected = value ? byId.get(value) : undefined;

  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const filtered: PickableRecord[] = useMemo(() => {
    const all = records.data ?? [];
    const q = query.trim().toLowerCase();
    return q ? all.filter((r) => r.label.toLowerCase().includes(q)) : all;
  }, [records.data, query]);

  useEffect(() => {
    if (!open) return;
    function onDocMouseDown(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", onDocMouseDown);
    return () => document.removeEventListener("mousedown", onDocMouseDown);
  }, [open]);

  function pick(record: PickableRecord) {
    onChange(record.id);
    setQuery("");
    setOpen(false);
  }

  function onKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (!open && (e.key === "ArrowDown" || e.key === "Enter")) {
      setOpen(true);
      setActiveIndex(0);
      return;
    }
    if (!open) return;
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActiveIndex((i) => Math.min(i + 1, filtered.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActiveIndex((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const chosen = filtered[activeIndex];
      if (chosen) pick(chosen);
    } else if (e.key === "Escape") {
      setOpen(false);
    }
  }

  const activeId = filtered[activeIndex] ? `lookup-option-${filtered[activeIndex].id}` : undefined;

  return (
    <div className="lookup-picker" ref={containerRef}>
      <div className="lookup-picker-input-row">
        <input
          type="text"
          role="combobox"
          aria-expanded={open}
          aria-controls="lookup-picker-listbox"
          aria-activedescendant={activeId}
          value={open ? query : selected?.label ?? ""}
          placeholder={records.isLoading ? "Loading..." : placeholder || "Search..."}
          disabled={disabled}
          required={required && !value}
          onFocus={() => {
            setOpen(true);
            setQuery("");
            setActiveIndex(0);
          }}
          onChange={(e) => {
            setQuery(e.target.value);
            setActiveIndex(0);
            setOpen(true);
          }}
          onKeyDown={onKeyDown}
        />
        {value && !disabled && (
          <button type="button" className="lookup-picker-clear" aria-label="Clear selection" onClick={() => onChange("")}>
            ×
          </button>
        )}
      </div>
      {open && (
        <ul className="lookup-picker-menu" id="lookup-picker-listbox" role="listbox">
          {filtered.length === 0 && <li className="lookup-picker-option-empty">No matching records</li>}
          {filtered.map((r, i) => (
            <li
              key={r.id}
              id={`lookup-option-${r.id}`}
              role="option"
              aria-selected={r.id === value}
              className={`lookup-picker-option${i === activeIndex ? " active" : ""}`}
              onMouseEnter={() => setActiveIndex(i)}
              onMouseDown={(e) => {
                e.preventDefault(); // keep focus/avoid blur-before-click
                pick(r);
              }}
            >
              {r.label}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
