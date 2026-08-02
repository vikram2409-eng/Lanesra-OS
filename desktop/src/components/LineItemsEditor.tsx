import { centsToInputValue, formatCents, parseDecimalToCents } from "../lib/money";
import { computeDocumentTotals, computeLineTotal, type LineInput } from "../lib/lineCalc";
import type { Product } from "../lib/types";

export function LineItemsEditor({
  lines,
  onChange,
  products,
  currencyCode,
}: {
  lines: LineInput[];
  onChange: (lines: LineInput[]) => void;
  products: Product[];
  currencyCode: string;
}) {
  function update(index: number, patch: Partial<LineInput>) {
    const next = lines.slice();
    next[index] = { ...next[index], ...patch };
    onChange(next);
  }

  function addLine() {
    onChange([
      ...lines,
      { product_id: null, description: "", quantity_milli: 1000, unit_price_cents: 0, discount_bp: 0, tax_rate_bp: 0 },
    ]);
  }

  function removeLine(index: number) {
    onChange(lines.filter((_, i) => i !== index));
  }

  function applyProduct(index: number, productId: string) {
    const product = products.find((p) => p.id === productId);
    if (!product) {
      update(index, { product_id: null });
      return;
    }
    update(index, {
      product_id: product.id,
      description: product.name,
      unit_price_cents: product.unit_price_cents,
      tax_rate_bp: product.tax_rate_bp,
      quantity_milli: product.default_quantity_milli,
    });
  }

  const totals = computeDocumentTotals(lines);

  return (
    <div className="card line-items-table">
      <table>
        <thead>
          <tr>
            <th>Product</th>
            <th>Description</th>
            <th>Qty</th>
            <th>Unit price</th>
            <th>Discount %</th>
            <th>Tax %</th>
            <th>Line total</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {lines.map((line, idx) => (
            <tr key={idx}>
              <td>
                <select value={line.product_id ?? ""} onChange={(e) => applyProduct(idx, e.target.value)}>
                  <option value="">Custom</option>
                  {products.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name}
                    </option>
                  ))}
                </select>
              </td>
              <td>
                <input
                  value={line.description}
                  onChange={(e) => update(idx, { description: e.target.value })}
                  required
                />
              </td>
              <td>
                <input
                  type="number"
                  step="0.001"
                  style={{ width: 70 }}
                  value={(line.quantity_milli / 1000).toString()}
                  onChange={(e) => update(idx, { quantity_milli: Math.round(parseFloat(e.target.value || "0") * 1000) })}
                />
              </td>
              <td>
                <input
                  type="number"
                  step="0.01"
                  style={{ width: 90 }}
                  value={centsToInputValue(line.unit_price_cents)}
                  onChange={(e) => update(idx, { unit_price_cents: parseDecimalToCents(e.target.value) })}
                />
              </td>
              <td>
                <input
                  type="number"
                  step="0.01"
                  style={{ width: 70 }}
                  value={(line.discount_bp / 100).toString()}
                  onChange={(e) => update(idx, { discount_bp: Math.round(parseFloat(e.target.value || "0") * 100) })}
                />
              </td>
              <td>
                <input
                  type="number"
                  step="0.01"
                  style={{ width: 70 }}
                  value={(line.tax_rate_bp / 100).toString()}
                  onChange={(e) => update(idx, { tax_rate_bp: Math.round(parseFloat(e.target.value || "0") * 100) })}
                />
              </td>
              <td>{formatCents(computeLineTotal(line), currencyCode)}</td>
              <td>
                <button type="button" className="btn btn-danger" onClick={() => removeLine(idx)}>
                  ✕
                </button>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <button type="button" className="btn" onClick={addLine} style={{ marginTop: 8 }}>
        + Add line
      </button>
      <div style={{ marginTop: 12, textAlign: "right", fontSize: 14 }}>
        <div>Subtotal: {formatCents(totals.subtotal_cents, currencyCode)}</div>
        <div>Discount: -{formatCents(totals.discount_cents, currencyCode)}</div>
        <div>Tax: {formatCents(totals.tax_cents, currencyCode)}</div>
        <div style={{ fontWeight: 700 }}>Total: {formatCents(totals.total_cents, currencyCode)}</div>
      </div>
    </div>
  );
}
