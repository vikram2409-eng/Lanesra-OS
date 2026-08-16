import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { centsToInputValue, formatCents, parseDecimalToCents } from "../../lib/money";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { useCustomFieldElements } from "../../components/CustomFieldsSection";
import { LayoutFormFields } from "../../components/LayoutFormFields";
import { LayoutDetailFields } from "../../components/LayoutDetailFields";
import { CustomFieldsCard } from "../../components/CustomFieldsCard";
import { AuditByline, AuditTrail } from "../../components/AuditTrail";
import { CustomFieldFilterBar } from "../../components/CustomFieldFilterBar";
import type { Prefill, Section } from "../../components/AppShell";
import { PRODUCT_TYPES, type CustomFieldValues, type Product, type ProductInput } from "../../lib/types";
import { useCustomFieldFilters } from "../../lib/useCustomFieldFilters";
import { useCanWriteObject } from "../../lib/useCanWriteObject";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string } | { mode: "detail"; id: string };

const PRODUCT_EXPORT_COLUMNS = [
  { label: "Number", get: (p: Product) => p.product_number },
  { label: "SKU", get: (p: Product) => p.sku ?? "" },
  { label: "Type", get: (p: Product) => p.type },
  { label: "Name", get: (p: Product) => p.name },
  { label: "Category", get: (p: Product) => p.category ?? "" },
  { label: "Description", get: (p: Product) => p.description ?? "" },
  { label: "Unit price (cents)", get: (p: Product) => String(p.unit_price_cents) },
  { label: "Cost (cents)", get: (p: Product) => String(p.cost_cents) },
  { label: "Tax rate (bp)", get: (p: Product) => String(p.tax_rate_bp) },
  { label: "Active", get: (p: Product) => (p.is_active ? "Yes" : "No") },
];

const emptyInput: ProductInput = {
  sku: null,
  type: "Service",
  name: "",
  category: null,
  description: null,
  unit_price_cents: 0,
  cost_cents: 0,
  tax_rate_bp: 0,
  default_quantity_milli: 1000,
  is_active: true,
};

export function Products({
  prefill,
  onPrefillConsumed,
}: {
  prefill?: Prefill | null;
  onPrefillConsumed?: () => void;
  onNavigateTo?: (section: Section, prefill: Prefill) => void;
} = {}) {
  const [view, setView] = useState<View>(() => (prefill?.openId ? { mode: "detail", id: prefill.openId } : { mode: "list" }));
  const queryClient = useQueryClient();
  const products = useQuery({ queryKey: ["products"], queryFn: () => api.listProducts() });
  const fieldFilters = useCustomFieldFilters("Product");
  const canWrite = useCanWriteObject("Product");

  useEffect(() => {
    if (prefill?.openId) onPrefillConsumed?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["products"] });
  }

  if (view.mode === "create" || view.mode === "edit") {
    return (
      <ProductForm
        productId={view.mode === "edit" ? view.id : undefined}
        onDone={() => {
          invalidate();
          setView({ mode: "list" });
        }}
        onCancel={() => setView({ mode: "list" })}
      />
    );
  }

  if (view.mode === "detail") {
    return (
      <ProductDetail
        id={view.id}
        onEdit={() => setView({ mode: "edit", id: view.id })}
        onBack={() => setView({ mode: "list" })}
      />
    );
  }

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Products &amp; Services</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton rows={products.data ?? []} columns={PRODUCT_EXPORT_COLUMNS} filename="products.csv" />
          <button
            className="btn btn-primary"
            onClick={() => setView({ mode: "create" })}
            disabled={!canWrite}
            title={canWrite ? undefined : "You have view-only access to Products through an app"}
          >
            + New product
          </button>
        </div>
      </div>
      <CustomFieldFilterBar filters={fieldFilters} />
      {products.isLoading && <p>Loading...</p>}
      {products.data && products.data.length === 0 && <p className="empty-state">No products yet.</p>}
      {products.data && products.data.length > 0 && (() => {
        const rows = products.data.filter((p) => fieldFilters.matches(p.id));
        return rows.length === 0 ? (
          <p className="empty-state">No products match the current filters.</p>
        ) : (
        <table>
          <thead>
            <tr>
              <th>Number</th>
              <th>Name</th>
              <th>Type</th>
              <th>Unit price</th>
              <th>Active</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {rows.map((p) => (
              <tr key={p.id} onClick={() => setView({ mode: "detail", id: p.id })} style={{ cursor: "pointer" }}>
                <td><span className="id-link">{p.product_number}</span></td>
                <td>{p.name}</td>
                <td>{p.type}</td>
                <td>{formatCents(p.unit_price_cents)}</td>
                <td>{p.is_active ? "Yes" : "No"}</td>
                <td>
                  <button
                    className="btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      setView({ mode: "edit", id: p.id });
                    }}
                    disabled={!canWrite}
                    title={canWrite ? undefined : "You have view-only access to Products through an app"}
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

function ProductForm({
  productId,
  onDone,
  onCancel,
}: {
  productId?: string;
  onDone: () => void;
  onCancel: () => void;
}) {
  const existing = useQuery({
    queryKey: ["product", productId],
    queryFn: () => api.getProduct(productId as string),
    enabled: !!productId,
  });
  const existingCustomFields = useQuery({
    queryKey: ["customFieldValues", productId],
    queryFn: () => api.getCustomFieldValues(productId as string),
    enabled: !!productId,
  });
  const [input, setInput] = useState<ProductInput>(emptyInput);
  const [customValues, setCustomValues] = useState<CustomFieldValues>({});
  const [loadedFor, setLoadedFor] = useState<string | undefined>(undefined);
  const [error, setError] = useState<string | null>(null);

  if (existing.data && existingCustomFields.data !== undefined && loadedFor !== productId) {
    const { sku, type, name, category, description, unit_price_cents, cost_cents, tax_rate_bp, default_quantity_milli, is_active } =
      existing.data;
    setInput({ sku, type, name, category, description, unit_price_cents, cost_cents, tax_rate_bp, default_quantity_milli, is_active });
    setCustomValues(existingCustomFields.data);
    setLoadedFor(productId);
  }

  const save = useMutation({
    mutationFn: async () => {
      const product = productId ? await api.updateProduct(productId, input) : await api.createProduct(input);
      const ruleMessages = await api.setCustomFieldValues("Product", product.id, customValues);
      showRuleMessages(ruleMessages);
      return product;
    },
    onSuccess: onDone,
    onError: (err) => setError(err instanceof ApiError ? err.message : "Could not save the product"),
  });

  const { order: customFieldOrder, elements: customFieldElements } = useCustomFieldElements({
    entityType: "Product",
    status: input.is_active ? "true" : "false",
    values: customValues,
    onChange: setCustomValues,
  });

  return (
    <div>
      <h2>{productId ? "Edit product" : "New product"}</h2>
      {error && <div className="error-banner">{error}</div>}
      <form
        className="form-grid"
        onSubmit={(e) => {
          e.preventDefault();
          save.mutate();
        }}
      >
        <LayoutFormFields
          entityType="Product"
          order={["name", "type", "sku", "unit_price_cents", "tax_rate_bp", "is_active", "description", ...customFieldOrder]}
          fields={{
            name: (
              <div className="form-field full" key="name">
                <label>Name</label>
                <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
              </div>
            ),
            type: (
              <div className="form-field" key="type">
                <label>Type</label>
                <select value={input.type} onChange={(e) => setInput({ ...input, type: e.target.value })}>
                  {PRODUCT_TYPES.map((t) => (
                    <option key={t} value={t}>
                      {t}
                    </option>
                  ))}
                </select>
              </div>
            ),
            sku: (
              <div className="form-field" key="sku">
                <label>SKU / code</label>
                <input value={input.sku ?? ""} onChange={(e) => setInput({ ...input, sku: e.target.value || null })} />
              </div>
            ),
            unit_price_cents: (
              <div className="form-field" key="unit_price_cents">
                <label>Unit price</label>
                <input
                  type="number"
                  step="0.01"
                  value={centsToInputValue(input.unit_price_cents)}
                  onChange={(e) => setInput({ ...input, unit_price_cents: parseDecimalToCents(e.target.value) })}
                />
              </div>
            ),
            tax_rate_bp: (
              <div className="form-field" key="tax_rate_bp">
                <label>Tax rate (%)</label>
                <input
                  type="number"
                  step="0.01"
                  value={(input.tax_rate_bp / 100).toString()}
                  onChange={(e) => setInput({ ...input, tax_rate_bp: Math.round(parseFloat(e.target.value || "0") * 100) })}
                />
              </div>
            ),
            is_active: (
              <div className="form-field" key="is_active">
                <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
                  <input
                    type="checkbox"
                    checked={input.is_active}
                    onChange={(e) => setInput({ ...input, is_active: e.target.checked })}
                  />
                  Active
                </label>
              </div>
            ),
            description: (
              <div className="form-field full" key="description">
                <label>Description</label>
                <textarea
                  value={input.description ?? ""}
                  onChange={(e) => setInput({ ...input, description: e.target.value || null })}
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
    </div>
  );
}

/**
 * Record-detail-page round: Products previously had no detail view at all -
 * list row click and menu went straight to Edit. No related-records list
 * here (unlike Company/Contact/Quote/Order/Invoice/Contract/Task) - which
 * quotes/orders/invoices reference this product lives in each document's
 * own line items, not a query this screen can cheaply run client-side, so
 * this stays a focused field overview instead of guessing at one.
 *
 * Screen/App Builder Phase 4: the built-in fields below go through
 * `LayoutDetailFields` instead of a hardcoded list, so the same layout
 * that already controls ProductForm's field order/columns also controls
 * this read-only view. Custom fields keep their own always-editable
 * `CustomFieldsCard` below, unchanged - that's a separate, pre-existing
 * mechanism this phase doesn't touch.
 */
function ProductDetail({ id, onEdit, onBack }: { id: string; onEdit: () => void; onBack: () => void }) {
  const canWrite = useCanWriteObject("Product");
  const product = useQuery({ queryKey: ["product", id], queryFn: () => api.getProduct(id) });

  if (!product.data) return <p>Loading...</p>;
  const p = product.data;

  return (
    <div>
      <div className="toolbar">
        <button className="btn" onClick={onBack}>
          ← Back
        </button>
        <button className="btn" onClick={onEdit} disabled={!canWrite} title={canWrite ? undefined : "You have view-only access to Products through an app"}>
          Edit
        </button>
      </div>
      <h2>
        {p.name} <span className={`badge${p.is_active ? " badge-success" : ""}`}>{p.is_active ? "Active" : "Inactive"}</span>
      </h2>
      <p style={{ color: "var(--text-muted)" }}>
        {p.product_number} · {p.type}
      </p>
      <AuditByline createdAt={p.created_at} createdBy={p.created_by} updatedAt={p.updated_at} updatedBy={p.updated_by} />

      <div className="card">
        <h3 style={{ marginTop: 0 }}>Details</h3>
        <div className="form-grid">
        <LayoutDetailFields
          entityType="Product"
          order={["sku", "category", "unit_price_cents", "cost_cents", "tax_rate_bp", "description"]}
          fields={{
            sku: (
              <div className="form-field" key="sku">
                <label>SKU</label>
                <div>{p.sku ?? "—"}</div>
              </div>
            ),
            category: (
              <div className="form-field" key="category">
                <label>Category</label>
                <div>{p.category ?? "—"}</div>
              </div>
            ),
            unit_price_cents: (
              <div className="form-field" key="unit_price_cents">
                <label>Unit price</label>
                <div>{formatCents(p.unit_price_cents)}</div>
              </div>
            ),
            cost_cents: (
              <div className="form-field" key="cost_cents">
                <label>Cost</label>
                <div>{formatCents(p.cost_cents)}</div>
              </div>
            ),
            tax_rate_bp: (
              <div className="form-field" key="tax_rate_bp">
                <label>Tax rate</label>
                <div>{(p.tax_rate_bp / 100).toFixed(2)}%</div>
              </div>
            ),
            description: (
              <div className="form-field full" key="description">
                <label>Description</label>
                <div>{p.description ?? "—"}</div>
              </div>
            ),
          }}
        />
        </div>
      </div>

      <div style={{ marginTop: 16 }}>
        <CustomFieldsCard entityType="Product" entityId={p.id} status={p.is_active ? "true" : "false"} />
        <AuditTrail entityType="Product" entityId={p.id} />
      </div>
    </div>
  );
}
