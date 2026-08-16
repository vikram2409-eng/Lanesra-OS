import { Fragment, useState } from "react";
import type { ReactNode } from "react";

import { useEffectiveLayout } from "../lib/useEffectiveLayout";

/**
 * Arranges a record form's fields per that entity's effective Screen
 * layout (Screen/App Builder Phase 1) - tabs of field sections, resolved
 * server-side against the signed-in user's roles. `fields` maps each
 * field's layout key to its already-built `.form-field` (or
 * `.form-field.full`) element; `order` is the form's own natural field
 * order, used both as the fallback when no layout is published yet for
 * this entity type and as a safety net for any key the layout doesn't
 * mention (appended to a trailing "Other fields" section) - a layout
 * built before a field existed, or one whose admin never got around to
 * placing every field, never hides it from the form.
 *
 * Expects to sit directly inside a `.form-grid` container, same as
 * `CustomFieldsSection` - it renders `.form-field`/`.form-grid.full`
 * elements, not a wrapper of its own, so the fallback path (still the
 * common case: most entities have no published layout yet) lays out
 * pixel-identical to every form before this component existed.
 */
export function LayoutFormFields({
  entityType,
  fields,
  order,
}: {
  entityType: string;
  fields: Record<string, ReactNode>;
  order: string[];
}) {
  const layout = useEffectiveLayout(entityType);
  const [activeIdx, setActiveIdx] = useState(0);
  const tabs = layout.data?.tabs?.tabs;

  if (!tabs || tabs.length === 0) {
    return (
      <>
        {order.map((key) => (
          <Fragment key={key}>{fields[key] ?? null}</Fragment>
        ))}
      </>
    );
  }

  const placed = new Set(tabs.flatMap((t) => t.sections.flatMap((s) => s.fields)));
  const leftover = order.filter((key) => !placed.has(key) && fields[key]);
  const effectiveTabs =
    leftover.length === 0
      ? tabs
      : [
          ...tabs.slice(0, -1),
          {
            ...tabs[tabs.length - 1],
            sections: [
              ...tabs[tabs.length - 1].sections,
              { id: "__leftover", title: "Other fields", fields: leftover },
            ],
          },
        ];

  const idx = Math.min(activeIdx, effectiveTabs.length - 1);
  const active = effectiveTabs[idx];
  const showSectionTitles = effectiveTabs.length > 1 || active.sections.length > 1;

  return (
    <>
      {effectiveTabs.length > 1 && (
        <div className="form-field full" style={{ marginBottom: 0 }}>
          <div className="tab-row" style={{ margin: 0 }}>
            {effectiveTabs.map((t, i) => (
              <button type="button" key={t.id} className={`tab${i === idx ? " active" : ""}`} onClick={() => setActiveIdx(i)}>
                {t.title}
              </button>
            ))}
          </div>
        </div>
      )}
      {active.sections.map((section) => (
        <div className="form-grid full" key={section.id} style={{ maxWidth: "none" }}>
          {showSectionTitles && (
            <div className="form-field full" style={{ marginBottom: -4 }}>
              <strong style={{ fontSize: 13 }}>{section.title}</strong>
            </div>
          )}
          {section.fields.map((key) => (
            <Fragment key={key}>{fields[key] ?? null}</Fragment>
          ))}
        </div>
      ))}
    </>
  );
}
