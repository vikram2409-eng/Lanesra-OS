import { Fragment, cloneElement, isValidElement, useState } from "react";
import type { CSSProperties, ReactNode } from "react";

import { useEffectiveLayout } from "../lib/useEffectiveLayout";
import { RelatedRecordsCard } from "./RelatedRecordsCard";
import type { LayoutSection } from "../lib/types";

/** Forces a field's own grid placement to match the section's admin-set
 * `full_width` choice for it, overriding whatever `.form-field`/
 * `.form-field.full` class the form's own JSX used - inline style always
 * wins over a class, in either direction, so this works whether the
 * layout is making a field wider or narrower than the code originally
 * wrote it. `undefined` fields (a stale key with nothing in `fields`)
 * pass through as `null`. */
function withSpan(node: ReactNode, fullWidth: boolean): ReactNode {
  if (!isValidElement(node)) return node ?? null;
  const existingStyle = (node.props as { style?: CSSProperties }).style ?? {};
  return cloneElement(node as React.ReactElement<{ style?: CSSProperties }>, {
    style: { ...existingStyle, gridColumn: fullWidth ? "1 / -1" : "auto" },
  });
}

/** A field the layout doesn't mention keeps whatever width the form's own
 * JSX gave it (`.form-field.full` vs plain `.form-field`) rather than
 * being forced to a single column - it's landing in the "Other fields"
 * safety net, not something an admin actually placed, so there's no
 * admin intent to honor instead. */
function inferFullWidth(node: ReactNode): boolean {
  if (!isValidElement(node)) return false;
  const className = (node.props as { className?: string }).className ?? "";
  return className.split(/\s+/).includes("full");
}

/**
 * Arranges a record form's fields - and, once it has a saved record to
 * point at, its related-records lists - per that entity's effective
 * Screen layout (Screen/App Builder Phases 1-3): tabs of field sections,
 * each laid out in its own `columns`-wide grid (Phase 2), plus whichever
 * relationships (Phase 3) an admin placed on each tab, all resolved
 * server-side against the signed-in user's roles.
 *
 * `fields` maps each field's layout key to its already-built
 * `.form-field` (or `.form-field.full`) element; `order` is the form's
 * own natural field order, used both as the fallback when no layout is
 * published yet for this entity type and as a safety net for any key the
 * layout doesn't mention (appended to a trailing "Other fields" section)
 * - a layout built before a field existed, or one whose admin never got
 * around to placing every field, never hides it from the form.
 *
 * `entityId`/`relatedKeys` are optional and only meaningful once editing
 * an existing record (a create form has nothing to link related records
 * to yet): `relatedKeys` is every `RelationshipDefinition.key` applicable
 * to this entity type, the caller's full assignable set, the same role
 * `order` plays for fields. A key no tab claims still isn't hidden - it
 * renders in an always-visible `RelatedRecordsCard` outside the tab
 * strip, the same safety net unplaced fields get.
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
  entityId,
  relatedKeys = [],
}: {
  entityType: string;
  fields: Record<string, ReactNode>;
  order: string[];
  entityId?: string;
  relatedKeys?: string[];
}) {
  const layout = useEffectiveLayout(entityType);
  const [activeIdx, setActiveIdx] = useState(0);
  const tabs = layout.data?.tabs?.tabs;
  const showRelated = !!entityId && relatedKeys.length > 0;

  if (!tabs || tabs.length === 0) {
    return (
      <>
        {order.map((key) => (
          <Fragment key={key}>{fields[key] ?? null}</Fragment>
        ))}
        {showRelated && (
          <div className="form-field full">
            <RelatedRecordsCard entityType={entityType} entityId={entityId as string} />
          </div>
        )}
      </>
    );
  }

  const placed = new Set(tabs.flatMap((t) => t.sections.flatMap((s) => s.fields.map((f) => f.key))));
  const leftover = order.filter((key) => !placed.has(key) && fields[key]);
  const leftoverSection: LayoutSection = {
    id: "__leftover",
    title: "Other fields",
    columns: 2,
    fields: leftover.map((key) => ({ key, full_width: inferFullWidth(fields[key]) })),
  };
  const effectiveTabs =
    leftover.length === 0
      ? tabs
      : [...tabs.slice(0, -1), { ...tabs[tabs.length - 1], sections: [...tabs[tabs.length - 1].sections, leftoverSection] }];

  const claimedRelated = new Set(tabs.flatMap((t) => t.related));
  const leftoverRelated = relatedKeys.filter((k) => !claimedRelated.has(k));

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
        <div
          className="form-grid full"
          key={section.id}
          style={{ maxWidth: "none", gridTemplateColumns: `repeat(${section.columns}, 1fr)` }}
        >
          {showSectionTitles && (
            <div className="form-field full" style={{ marginBottom: -4 }}>
              <strong style={{ fontSize: 13 }}>{section.title}</strong>
            </div>
          )}
          {section.fields.map((f) => (
            <Fragment key={f.key}>{withSpan(fields[f.key] ?? null, f.full_width)}</Fragment>
          ))}
        </div>
      ))}
      {showRelated && active.related.length > 0 && (
        <div className="form-field full">
          <RelatedRecordsCard entityType={entityType} entityId={entityId as string} only={active.related} />
        </div>
      )}
      {showRelated && leftoverRelated.length > 0 && (
        <div className="form-field full">
          <RelatedRecordsCard entityType={entityType} entityId={entityId as string} only={leftoverRelated} />
        </div>
      )}
    </>
  );
}
