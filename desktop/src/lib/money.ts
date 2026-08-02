// Presentation-only helpers mirroring src-tauri/src/domain/money.rs. All
// arithmetic that gets persisted happens in Rust; this file only formats
// already-computed integer cents for display and parses user decimal input
// back into integer cents before it is sent to a command.

export function formatCents(cents: number, currencyCode = "USD"): string {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency: currencyCode,
  }).format(cents / 100);
}

export function centsToInputValue(cents: number): string {
  return (cents / 100).toFixed(2);
}

export function parseDecimalToCents(value: string): number {
  const normalized = value.trim().replace(/,/g, "");
  const asNumber = Number.parseFloat(normalized || "0");
  if (Number.isNaN(asNumber)) return 0;
  return Math.round(asNumber * 100);
}

export function quantityMilliToInputValue(milli: number): string {
  return (milli / 1000).toString();
}

export function parseQuantityToMilli(value: string): number {
  const asNumber = Number.parseFloat(value || "0");
  if (Number.isNaN(asNumber)) return 0;
  return Math.round(asNumber * 1000);
}

export function bpToPercentInputValue(bp: number): string {
  return (bp / 100).toString();
}

export function parsePercentToBp(value: string): number {
  const asNumber = Number.parseFloat(value || "0");
  if (Number.isNaN(asNumber)) return 0;
  return Math.round(asNumber * 100);
}
