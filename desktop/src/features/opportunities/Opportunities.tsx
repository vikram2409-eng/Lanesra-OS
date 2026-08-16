import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { formatCents, centsToInputValue, parseDecimalToCents } from "../../lib/money";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { useCustomFieldElements } from "../../components/CustomFieldsSection";
import { LayoutFormFields } from "../../components/LayoutFormFields";
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import { AuditByline, AuditTrail } from "../../components/AuditTrail";
import type { Prefill } from "../../components/AppShell";
import { useCustomFieldFilters } from "../../lib/useCustomFieldFilters";
import { useCanWriteObject } from "../../lib/useCanWriteObject";
import {
  OPPORTUNITY_STAGES,
  OPPORTUNITY_STATUSES,
  type CustomFieldValues,
  type Opportunity,
  type OpportunityInput,
} from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

function opportunityExportColumns(companyNameById: Map<string, string>) {
  return [
    { label: "Number", get: (o: Opportunity) => o.opportunity_number },
    { label: "Name", get: (o: Opportunity) => o.name },
    { label: "Company", get: (o: Opportunity) => companyNameById.get(o.company_id) ?? "" },
    { label: "Stage", get: (o: Opportunity) => o.stage },
    { label: "Status", get: (o: Opportunity) => o.status },
    { label: "Value (cents)", get: (o: Opportunity) => String(o.value_cents) },
    { label: "Currency", get: (o: Opportunity) => o.currency_code },
    { label: "Probability (bp)", get: (o: Opportunity) => String(o.probability_bp) },
    { label: "Expected close date", get: (o: Opportunity) => o.expected_close_date ?? "" },
  ];
}

function emptyInput(companyId: string, currency: string): OpportunityInput {
  return {
    company_id: companyId,
    primary_contact_id: null,
    name: "",
    stage: "New",
    status: "Open",
    value_cents: 0,
    currency_code: currency,
    probability_bp: 1000,
    expected_close_date: null,
    owner_user_id: null,
    lost_reason: null,
    next_step: null,
  };
}

export function Opportunities({
  prefill,
  onPrefillConsumed,
}: { prefill?: Prefill | null; onPrefillConsumed?: () => void } = {}) {
  const [view, setView] = useState<View>(() =>
    prefill?.openId ? { mode: "edit", id: prefill.openId } : prefill?.companyId ? { mode: "create" } : { mode: "list" },
  );
  const queryClient = useQueryClient();
  const opportunities = useQuery({ queryKey: ["opportunities"], queryFn: () => api.listOpportunities() });
  const fieldFilters = useCustomFieldFilters("Opportunity");
  const canWrite = useCanWriteObject("Opportunity");
  const companies = useQuery({ queryKey: ["companies"], queryFn: () => api.listCompanies() });

  // One-shot: this component fully remounts on every navigation into this
  // section, so "on mount" reliably means "just navigated here" - see
  // Prefill's doc comment in AppShell.tsx.
  useEffect(() => {
    if (prefill?.companyId || prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["opportunities"] });
  }

  if (view.mode !== "list") {
    return (
      <OpportunityForm
        opportunityId={view.mode === "edit" ? view.id : undefined}
        companies={companies.data ?? []}
        initialCompanyId={view.mode === "create" ? prefill?.companyId : undefined}
        initialContactId={view.mode === "create" ? prefill?.contactId : undefined}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  const companyNameById = new Map((companies.data ?? []).map((c) => [c.id, c.name]));

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Sales Pipeline</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton
            rows={opportunities.data ?? []}
            columns={opportunityExportColumns(companyNameById)}
            filename="opportunities.csv"
          />
          <button
            className="btn btn-primary"
            onClick={() => setView({ mode: "create" })}
            disabled={!canWrite}
            title={canWrite ? undefined : "You have view-only access to Opportunities through an app"}
          >
            + New opportunity
          </button>
        </div>
      </div>
      <CustomFieldFilterBar filters={fieldFilters} />
      {opportunities.isLoading && <p>Loading...</p>}
      {opportunities.data && opportunities.data.length === 0 && (
        <p className="empty-state">No opportunities yet.</p>
      )}
      {opportunities.data && opportunities.data.length > 0 && (() => {
        const rows = opportunities.data.filter((o) => fieldFilters.matches(o.id));
        return rows.length === 0 ? (
          <p className="empty-state">No opportunities match the current filters.</p>
        ) : (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Name</th>
              <th>Company</th>
              <th>Stage</th>
              <th>Value</th>
              <th>Probability</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((o) => (
              <tr key={o.id}>
                <td>{o.opportunity_number}</td>
                <td>{o.name}</td>
                <td>{companyNameById.get(o.company_id) ?? "—"}</td>
                <td>{o.stage}</td>
                <td>{formatCents(o.value_cents, o.currency_code)}</td>
                <td>{(o.probability_bp / 100).toFixed(0)}%</td>
                <td>
                  <button
                    className="btn"
                    onClick={() => setView({ mode: "edit", id: o.id })}
                    disabled={!canWrite}
                    title={canWrite ? undefined : "You have view-only access to Opportunities through an app"}
                  >
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        );
      })()}
    </div>
  );
}

function OpportunityForm({
  opportunityId,
  companies,
  initialCompanyId,
  initialContactId,
  onDone,
  onCancel,
}: {
  opportunityId?: string;
  companies: { id: string; name: string; currency_code?: string }[];
  initialCompanyId?: string;
  initialContactId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["opportunity", opportunityId],
    queryFn: () => api.getOpportunity(opportunityId as string),
    enabled: !!opportunityId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", opportunityId],
    queryFn: () => api.getCustomFieldValues(opportunityId as string),
    enabled: !!opportunityId,
  });
  const [input, setInput] = useState<OpportunityInput>(() => {
    const base = emptyInput(initialCompanyId ?? companies[0]?.id ?? "", "USD");
    return initialContactId ? { ...base, primary_contact_id: initialContactId } : base;
  });
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  const contacts = useQuery({
    queryKey: ["contactsByCompany", input.company_id],
    queryFn: () => api.listContactsByCompany(input.company_id),
    enabled: !!input.company_id,
  });

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== opportunityId) {
    const {
      company_id,
      primary_contact_id,
      name,
      stage,
      status,
      value_cents,
      currency_code,
      probability_bp,
      expected_close_date,
      owner_user_id,
      lost_reason,
      next_step,
    } = existing.data;
    setInput({
      company_id,
      primary_contact_id,
      name,
      stage,
      status,
      value_cents,
      currency_code,
      probability_bp,
      expected_close_date,
      owner_user_id,
      lost_reason,
      next_step,
    });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(opportunityId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const opportunity = opportunityId
        ? await api.updateOpportunity(opportunityId, input)
        : await api.createOpportunity(input);
      const ruleMessages = await api.setCustomFieldValues("Opportunity", opportunity.id, customValues);
      showRuleMessages(ruleMessages);
      return opportunity;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the opportunity"),
  });

  const { order: customFieldOrder, elements: customFieldElements } = useCustomFieldElements({
    entityType: "Opportunity",
    status: input.status,
    values: customValues,
    onChange: setCustomValues,
  });

  return (
    <div>
      <h2>{opportunityId ? "Edit opportunity" : "New opportunity"}</h2>
      {existing.data && (
        <AuditByline
          createdAt={existing.data.created_at}
          createdBy={existing.data.created_by}
          updatedAt={existing.data.updated_at}
          updatedBy={existing.data.updated_by}
        />
      )}
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <LayoutFormFields
          entityType="Opportunity"
          order={[
            "name", "company_id", "primary_contact_id", "stage", "status",
            "value_cents", "probability_bp", "next_step", ...customFieldOrder,
          ]}
          fields={{
            name: (
              <div className="form-field full" key="name">
                <label>Name</label>
                <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
              </div>
            ),
            company_id: (
              <div className="form-field" key="company_id">
                <label>Company</label>
                <select
                  value={input.company_id}
                  onChange={(e) => setInput({ ...input, company_id: e.target.value, primary_contact_id: null })}
                  required
                >
                  {companies.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                    </option>
                  ))}
                </select>
              </div>
            ),
            primary_contact_id: (
              <div className="form-field" key="primary_contact_id">
                <label>Primary contact (optional)</label>
                <select
                  value={input.primary_contact_id ?? ""}
                  onChange={(e) => setInput({ ...input, primary_contact_id: e.target.value || null })}
                >
                  <option value="">— None —</option>
                  {(contacts.data ?? []).map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.first_name} {c.last_name}
                    </option>
                  ))}
                </select>
              </div>
            ),
            stage: (
              <div className="form-field" key="stage">
                <label>Stage</label>
                <select value={input.stage} onChange={(e) => setInput({ ...input, stage: e.target.value })}>
                  {OPPORTUNITY_STAGES.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </div>
            ),
            status: (
              <div className="form-field" key="status">
                <label>Status</label>
                <select value={input.status} onChange={(e) => setInput({ ...input, status: e.target.value })}>
                  {OPPORTUNITY_STATUSES.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </div>
            ),
            value_cents: (
              <div className="form-field" key="value_cents">
                <label>Value</label>
                <input
                  type="number"
                  step="0.01"
                  value={centsToInputValue(input.value_cents)}
                  onChange={(e) => setInput({ ...input, value_cents: parseDecimalToCents(e.target.value) })}
                />
              </div>
            ),
            probability_bp: (
              <div className="form-field" key="probability_bp">
                <label>Probability (%)</label>
                <input
                  type="number"
                  min={0}
                  max={100}
                  value={(input.probability_bp / 100).toString()}
                  onChange={(e) => setInput({ ...input, probability_bp: Math.round(parseFloat(e.target.value || "0") * 100) })}
                />
              </div>
            ),
            next_step: (
              <div className="form-field full" key="next_step">
                <label>Next step</label>
                <input
                  value={input.next_step ?? ""}
                  onChange={(e) => setInput({ ...input, next_step: e.target.value || null })}
                />
              </div>
            ),
            ...customFieldElements,
          }}
        />
        <div className="form-field full" style={{ flexDirection: "row", gap: 8 }}>
          <button className="btn btn-primary" type="submit" disabled={save.isPending}>
            Save
          </button>
          <button className="btn" type="button" onClick={onCancel}>
            Cancel
          </button>
        </div>
      </form>
      {opportunityId && <AuditTrail entityType="Opportunity" entityId={opportunityId} />}
    </div>
  );
}
