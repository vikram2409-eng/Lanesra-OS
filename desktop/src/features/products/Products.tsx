import { useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api, ApiError } from "../../lib/api";
import { showRuleMessages } from "../../lib/ruleMessages";
import { centsToInputValue, formatCents, parseDecimalToCents } from "../../lib/money";
import { ExportCsvButton } from "../../components/ExportCsvButton";
import { CustomFieldsSection } from "../../components/CustomFieldsSection";
import { PRODUCT_TYPES, type CustomFieldValues, type Product, type ProductInput } from "../../lib/types";

type View = { mode: "list" } | { mode: "create" } | { mode: "edit"; id: string };

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

export function Products() {
  const [view, setView] = useState<View>({ mode: "list" });
  const queryClient = useQueryClient();
  const products = useQuery({ queryKey: ["products"], queryFn: () => api.listProducts() });

  function invalidate() {
    queryClient.invalidateQueries({ queryKey: ["products"] });
  }

  if (view.mode !== "list") {
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

  return (
    <div>
      <div className="toolbar">
        <h2 style={{ margin: 0 }}>Products &amp; Services</h2>
        <div style={{ display: "flex", gap: 8 }}>
          <ExportCsvButton rows={products.data ?? []} columns={PRODUCT_EXPORT_COLUMNS} filename="products.csv" />
          <button className="btn btn-primary" onClick={() => setView({ mode: "create" })}>
            + New product
          </button>
        </div>
      </div>
      {products.isLoading && <p>Loading...</p>}
      {products.data && products.data.length === 0 && <p className="empty-state">No products yet.</p>}
      {products.data && products.data.length > 0 && (
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
            {products.data.map((p) => (
              <tr key={p.id}>
                <td>{p.product_number}</td>
                <td>{p.name}</td>
                <td>{p.type}</td>
                <td>{formatCents(p.unit_price_cents)}</td>
                <td>{p.is_active ? "Yes" : "No"}</td>
                <td>
                  <button className="btn" onClick={() => setView({ mode: "edit", id: p.id })}>
                    Edit
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
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
        <div className="form-field full">
          <label>Name</label>
          <input value={input.name} onChange={(e) => setInput({ ...input, name: e.target.value })} required />
        </div>
        <div className="form-field">
          <label>Type</label>
          <select value={input.type} onChange={(e) => setInput({ ...input, type: e.target.value })}>
            {PRODUCT_TYPES.map((t) => (
              <option key={t} value={t}>
                {t}
              </option>
            ))}
          </select>
        </div>
        <div className="form-field">
          <label>SKU / code</label>
          <input value={input.sku ?? ""} onChange={(e) => setInput({ ...input, sku: e.target.value || null })} />
        </div>
        <div className="form-field">
          <label>Unit price</label>
          <input
            type="number"
            step="0.01"
            value={centsToInputValue(input.unit_price_cents)}
            onChange={(e) => setInput({ ...input, unit_price_cents: parseDecimalToCents(e.target.value) })}
          />
        </div>
        <div className="form-field">
          <label>Tax rate (%)</label>
          <input
            type="number"
            step="0.01"
            value={(input.tax_rate_bp / 100).toString()}
            onChange={(e) => setInput({ ...input, tax_rate_bp: Math.round(parseFloat(e.target.value || "0") * 100) })}
          />
        </div>
        <div className="form-field">
          <label style={{ display: "flex", gap: 8, alignItems: "center" }}>
            <input
              type="checkbox"
              checked={input.is_active}
              onChange={(e) => setInput({ ...input, is_active: e.target.checked })}
            />
            Active
          </label>
        </div>
        <div className="form-field full">
          <label>Description</label>
          <textarea
            value={input.description ?? ""}
            onChange={(e) => setInput({ ...input, description: e.target.value || null })}
          />
        </div>
        <CustomFieldsSection
          entityType="Product"
          status={input.is_active ? "true" : "false"}
          values={customValues}
          onChange={setCustomValues}
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
