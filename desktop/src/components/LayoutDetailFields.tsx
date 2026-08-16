import { Fragment } from "react";
import type { ReactNode } from "react";

import { useEffectiveLayout } from "../lib/useEffectiveLayout";
import { RelatedRecordsCard } from "./RelatedRecordsCard";
import { inferFullWidth, withSpan } from "./LayoutFormFields";
import type { LayoutSection } from "../lib/types";

/**
 * Screen/App Builder Phase 4: the read-only counterpart to
 * `LayoutFormFields` - arranges a record's fields on its detail/Overview
 * view per that entity's effective Screen layout, same as the edit form
 * does, so an admin customizing a layout (adding a custom field, moving
 * a built-in one, changing column count) sees it reflected on both
 * without touching either screen's code. Before this, every detail page
 * hardcoded its own field list, so a custom field placed on the edit
 * form's layout was invisible here - Company/Contact's Overview never
 * showed custom fields at all.
 *
 * The layout's *tabs* are a form-building convenience (breaking a long
 * edit form into stages) that doesn't carry over to a detail view, which
 * is a single glanceable summary, not something filled out in steps - so
 * this flattens every tab's sections into one continuous read-only
 * sequence instead of reproducing the tab strip. A section's title still
 * shows as a sub-heading (when there's more than one), and its column
 * count and each field's full-width span still apply exactly as they do
 * in the edit form.
 *
 * Relationships work the same way: `relatedKeys` claimed by any tab
 * (Phase 3) and any left unclaimed both end up on the one always-visible
 * `RelatedRecordsCard` at the bottom - there's no tab to scope them to
 * here, so showing them all together is just the unfiltered behavior
 * every detail page already had before Phase 3 existed.
 *
 * `fields` maps each field's layout key to its already-built read-only
 * element (a `.form-field` with a `<label>` and a plain value, the
 * static counterpart of the `<input>` an edit form passes to
 * `LayoutFormFields`); `order` is the same fallback/safety-net role it
 * plays there. Expects to sit directly inside a container that doesn't
 * fight its own `.form-grid` elements, same as `LayoutFormFields`.
 */
export function LayoutDetailFields({
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

  const allSections = tabs.flatMap((t) => t.sections);
  const placed = new Set(allSections.flatMap((s) => s.fields.map((f) => f.key)));
  const leftover = order.filter((key) => !placed.has(key) && fields[key]);
  const leftoverSection: LayoutSection = {
    id: "__leftover",
    title: "Other fields",
    columns: 2,
    fields: leftover.map((key) => ({ key, full_width: inferFullWidth(fields[key]) })),
  };
  const sections = leftover.length === 0 ? allSections : [...allSections, leftoverSection];
  const showSectionTitles = sections.length > 1;

  return (
    <>
      {sections.map((section) => (
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
      {showRelated && (
        <div className="form-field full">
          <RelatedRecordsCard entityType={entityType} entityId={entityId as string} only={relatedKeys} />
        </div>
      )}
    </>
  );
}
