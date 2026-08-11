import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import { CustomFieldsSection } from "./CustomFieldsSection";
import { showRuleMessages } from "../lib/ruleMessages";
import type { CustomFieldEntityType, CustomFieldValues } from "../lib/types";

/**
 * Standalone "Custom fields" card for a document detail view (Quote,
 * Order, Invoice) that has no full edit form of its own - only a create
 * form and a detail/status-transition view. Renders nothing if no active
 * custom fields are defined for this entity type (same as
 * CustomFieldsSection itself), so it's safe to always mount.
 */
export function CustomFieldsCard({ entityType, entityId, status }: { entityType: CustomFieldEntityType; entityId: string; status: string }) {
  const queryClient = useQueryClient();
  const existing = useQuery({
    queryKey: ["customFieldValues", entityId],
    queryFn: () => api.getCustomFieldValues(entityId),
  });
  const defs = useQuery({
    queryKey: ["customFieldDefinitions", entityType],
    queryFn: () => api.listCustomFieldDefinitions(entityType, true),
  });
  const [values, setValues] = useState<CustomFieldValues>({});
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    if (existing.data !== undefined && !loaded) {
      setValues(existing.data);
      setLoaded(true);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [existing.data]);

  const save = useMutation({
    mutationFn: () => api.setCustomFieldValues(entityType, entityId, values),
    onSuccess: (messages) => {
      setError(null);
      setSuccess(true);
      queryClient.invalidateQueries({ queryKey: ["customFieldValues", entityId] });
      showRuleMessages(messages);
    },
    onError: (err) => {
      setSuccess(false);
      setError(err instanceof ApiError ? err.message : "Could not save custom fields");
    },
  });

  if (defs.data && defs.data.length === 0) return null;

  return (
    <div className="card">
      <h3 style={{ marginTop: 0 }}>Custom fields</h3>
      {error && <div className="error-banner">{error}</div>}
      {success && <div className="success-banner">Saved.</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          setSuccess(false);
          save.mutate();
        }}
      >
        <CustomFieldsSection entityType={entityType} status={status} values={values} onChange={setValues} />
        <div className="form-field full">
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save custom fields
          </button>
        </div>
      </form>
    </div>
  );
}
