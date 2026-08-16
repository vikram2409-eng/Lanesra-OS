import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

/** The layout a create/edit form should render for the signed-in user on
 * this entity type - the published layout whose roles include one of
 * theirs, or that object's Default if none do (see
 * `screen_layout_service::resolve_effective_layout` in the Rust core).
 * `data?.tabs` is null when no layout has ever been published for this
 * entity type, including the auto-provisioned Default (which starts
 * unpublished) - callers should fall back to their own hardcoded field
 * order in that case, exactly as every form behaved before this feature
 * existed. See `LayoutFormFields` for the shared fallback/render logic. */
export function useEffectiveLayout(entityType: string) {
  return useQuery({
    queryKey: ["effectiveScreenLayout", entityType],
    queryFn: () => api.effectiveScreenLayout(entityType),
  });
}
