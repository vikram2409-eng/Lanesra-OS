import { useQuery } from "@tanstack/react-query";

import { api } from "./api";

/** The signed-in user, for UI convenience checks (e.g. "is this an
 * Administrator, so show the Set-as-default control") - never itself the
 * security boundary, the same convention `useCanWriteObject` follows.
 * `App.tsx` already resolves this once at boot via a plain useState, not
 * react-query, so this is a second independent fetch rather than a shared
 * cache hit - cheap and already a real, existing command either way. */
export function useCurrentUser() {
  return useQuery({
    queryKey: ["currentUser"],
    queryFn: () => api.currentUser(),
  });
}

export function useIsAdmin(): boolean {
  const { data } = useCurrentUser();
  return data?.roles.includes("Administrator") ?? false;
}
