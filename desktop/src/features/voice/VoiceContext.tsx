import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";

/**
 * Voice-First Mode, PR 1 (spec §7 "Context-Aware Voice"): the one piece of
 * global state this feature needs that nothing in the app already tracked
 * (confirmed by direct investigation - `App.tsx` only ever held a local
 * `section` string and a one-shot `Prefill`, no persistent "what record is
 * the user looking at" signal). `VoiceModeButton` reads this to
 * auto-populate a command's context; every core record-detail view reports
 * itself in here via `useReportVoiceContext` on mount/id-change and clears
 * on unmount.
 */
interface VoiceContextValue {
  objectKey: string | null;
  recordId: string | null;
  setVoiceContext: (objectKey: string | null, recordId: string | null) => void;
}

const VoiceContextInternal = createContext<VoiceContextValue>({
  objectKey: null,
  recordId: null,
  setVoiceContext: () => {},
});

export function VoiceContextProvider({ children }: { children: ReactNode }) {
  const [objectKey, setObjectKey] = useState<string | null>(null);
  const [recordId, setRecordId] = useState<string | null>(null);
  const setVoiceContext = useCallback((ok: string | null, rid: string | null) => {
    setObjectKey(ok);
    setRecordId(rid);
  }, []);
  return (
    <VoiceContextInternal.Provider value={{ objectKey, recordId, setVoiceContext }}>{children}</VoiceContextInternal.Provider>
  );
}

export function useVoiceContext() {
  return useContext(VoiceContextInternal);
}

/** One line, added to a record-detail component's own body, reports it as
 * the active Voice context while it's open ("mark this Won" needs no name
 * repeated) and clears it again on navigating away. */
export function useReportVoiceContext(objectKey: string, recordId: string | null | undefined) {
  const { setVoiceContext } = useVoiceContext();
  useEffect(() => {
    if (!recordId) return;
    setVoiceContext(objectKey, recordId);
    return () => setVoiceContext(null, null);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [objectKey, recordId]);
}
