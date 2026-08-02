// Client-side preview mirroring src-tauri/src/domain/money.rs so forms can
// show running totals before submit. The backend recomputes authoritative
// totals server-side; this is presentation only.

export interface LineInput {
  product_id: string | null;
  description: string;
  quantity_milli: number;
  unit_price_cents: number;
  discount_bp: number;
  tax_rate_bp: number;
}

function extendQuantity(quantityMilli: number, unitPriceCents: number): number {
  return Math.floor((quantityMilli * unitPriceCents + 500) / 1000);
}

function applyBp(amountCents: number, bp: number): number {
  return Math.floor((amountCents * bp + 5000) / 10000);
}

export function computeLineTotal(line: LineInput): number {
  const gross = extendQuantity(line.quantity_milli, line.unit_price_cents);
  const discount = applyBp(gross, line.discount_bp);
  const net = gross - discount;
  const tax = applyBp(net, line.tax_rate_bp);
  return net + tax;
}

export function computeDocumentTotals(lines: LineInput[]) {
  let subtotal = 0;
  let discount = 0;
  let tax = 0;
  let total = 0;
  for (const line of lines) {
    const gross = extendQuantity(line.quantity_milli, line.unit_price_cents);
    const lineDiscount = applyBp(gross, line.discount_bp);
    const net = gross - lineDiscount;
    const lineTax = applyBp(net, line.tax_rate_bp);
    subtotal += gross;
    discount += lineDiscount;
    tax += lineTax;
    total += net + lineTax;
  }
  return { subtotal_cents: subtotal, discount_cents: discount, tax_cents: tax, total_cents: total };
}
