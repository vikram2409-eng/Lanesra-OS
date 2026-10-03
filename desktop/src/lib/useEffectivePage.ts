import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

/** The composed Page a record's detail view should render for the
 * signed-in user on this entity type, if any has ever been published -
 * mirrors `useEffectiveLayout` exactly (see its own doc comment), just
 * resolved against `page_layouts`/`PageDefinition` instead of
 * `screen_layouts`/`LayoutTabs`. `data?.page` is `null` whenever nothing
 * is published (including the common case: an entity type whose admin
 * never opened Page Builder at all) - every caller must treat that as
 * "render exactly as before," never as an error or an empty state. */
export function useEffectivePage(entityType: string) {
  return useQuery({
    queryKey: ["effectivePageLayout", entityType],
    queryFn: () => api.effectivePageLayout(entityType),
  });
}
