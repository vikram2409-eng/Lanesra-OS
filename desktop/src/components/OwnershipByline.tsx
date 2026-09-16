import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../lib/api";
import type { OwnerType } from "../lib/types";

/**
 * Enterprise Access Foundation, Phase 1 (spec §2): the Owner + Owning
 * Organization Unit byline for one record, alongside `AuditByline` on
 * every object's detail page. Every record gets a default owner (the
 * creating user) at create time - see ownership_service.rs's
 * `set_default_owner_on_create` - so this almost always has something to
 * show; `null` only happens for an Organization Owned/System Owned object,
 * which has no individual owner concept and renders nothing.
 *
 * Reassigning is gated behind the same placeholder "Assign" capability
 * check the backend enforces (Administrator-only, until Phase 2's real
 * Access Role engine ships) - a non-admin sees the current owner read-only
 * and a rejected mutation if they somehow still try.
 */
export function OwnershipByline({ objectKey, recordId }: { objectKey: string; recordId: string }) {
  const [reassigning, setReassigning] = useState(false);
  const queryClient = useQueryClient();

  const ownership = useQuery({
    queryKey: ["recordOwner", objectKey, recordId],
    queryFn: () => api.getRecordOwner(objectKey, recordId),
  });
  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const teams = useQuery({ queryKey: ["workTeams"], queryFn: () => api.listWorkTeams() });
  const orgUnits = useQuery({ queryKey: ["orgUnits"], queryFn: () => api.listOrgUnits() });

  if (ownership.isLoading) return null;
  if (!ownership.data || !ownership.data.owner) return null;

  const { owner, owning_org_unit_id } = ownership.data;
  const ownerName =
    owner.owner_type === "TEAM"
      ? teams.data?.find((t) => t.id === owner.owner_id)?.name ?? "Unknown team"
      : users.data?.find((u) => u.id === owner.owner_id)?.display_name ?? "Unknown user";
  const orgUnitName = orgUnits.data?.find((u) => u.id === owning_org_unit_id)?.name;

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["recordOwner", objectKey, recordId] });
  }

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap", margin: "2px 0 0" }}>
      <p style={{ color: "var(--text-muted)", fontSize: 12, margin: 0 }}>
        Owner: {ownerName}
        {orgUnitName && <> · {orgUnitName}</>}
      </p>
      {!reassigning && (
        <button className="link-button" style={{ fontSize: 12 }} onClick={() => setReassigning(true)}>
          Reassign
        </button>
      )}
      {reassigning && (
        <ReassignForm
          objectKey={objectKey}
          recordId={recordId}
          currentOwnerType={owner.owner_type as OwnerType}
          currentOwnerId={owner.owner_id}
          currentOrgUnitId={owning_org_unit_id}
          onDone={() => {
            invalidate();
            setReassigning(false);
          }}
          onCancel={() => setReassigning(false)}
        />
      )}
    </div>
  );
}

function ReassignForm({
  objectKey,
  recordId,
  currentOwnerType,
  currentOwnerId,
  currentOrgUnitId,
  onDone,
  onCancel,
}: {
  objectKey: string;
  recordId: string;
  currentOwnerType: OwnerType;
  currentOwnerId: string;
  currentOrgUnitId: string | null;
  onDone: () => void;
  onCancel: () => void;
}) {
  const [ownerType, setOwnerType] = useState<OwnerType>(currentOwnerType);
  const [ownerId, setOwnerId] = useState(currentOwnerId);
  const [orgUnitId, setOrgUnitId] = useState(currentOrgUnitId ?? "");
  const [error, setError] = useState<string | null>(null);

  const users = useQuery({ queryKey: ["users"], queryFn: () => api.listUsers() });
  const teams = useQuery({ queryKey: ["workTeams"], queryFn: () => api.listWorkTeams() });
  const orgUnits = useQuery({ queryKey: ["orgUnits"], queryFn: () => api.listOrgUnits() });

  const save = useMutation({
    mutationFn: () => api.setRecordOwner(objectKey, recordId, { owner_type: ownerType, owner_id: ownerId }, orgUnitId || null),
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not reassign this record"),
  });

  const ownerChoices = ownerType === "TEAM" ? teams.data?.filter((t) => t.can_own_records) ?? [] : users.data ?? [];

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6, flexWrap: "wrap" }}>
      {error && <span style={{ color: "var(--danger, #c00)", fontSize: 12 }}>{error}</span>}
      <select
        value={ownerType}
        onChange={(e) => {
          const next = e.target.value as OwnerType;
          setOwnerType(next);
          setOwnerId("");
        }}
        style={{ fontSize: 12 }}
      >
        <option value="USER">User</option>
        <option value="TEAM">Team</option>
      </select>
      <select value={ownerId} onChange={(e) => setOwnerId(e.target.value)} style={{ fontSize: 12 }}>
        <option value="">Select...</option>
        {ownerChoices.map((c) => (
          <option key={c.id} value={c.id}>
            {"display_name" in c ? c.display_name : c.name}
          </option>
        ))}
      </select>
      <select value={orgUnitId} onChange={(e) => setOrgUnitId(e.target.value)} style={{ fontSize: 12 }}>
        <option value="">No Organization Unit</option>
        {orgUnits.data?.map((u) => (
          <option key={u.id} value={u.id}>
            {"— ".repeat(u.depth)}
            {u.name}
          </option>
        ))}
      </select>
      <button className="btn" style={{ fontSize: 12, padding: "2px 8px" }} onClick={() => save.mutate()} disabled={!ownerId || save.isPending}>
        Save
      </button>
      <button className="btn" style={{ fontSize: 12, padding: "2px 8px" }} onClick={onCancel}>
        Cancel
      </button>
    </div>
  );
}
