//! Decimal-safe money and quantity arithmetic (BR-014). All persisted
//! amounts are integer minor units (cents); all persisted quantities and
//! rates are integers scaled by 1000 (milli) or 10000 (basis points).
//! Floating point is never used for values that get written to the
//! database - only i128 intermediates for overflow-safe multiplication,
//! rounded half-up back to i64.

pub type Cents = i64;

fn round_half_up(numerator: i128, denominator: i128) -> i64 {
    ((numerator + denominator / 2) / denominator) as i64
}

/// Multiplies a quantity (scaled by 1000) by a unit price in cents,
/// returning the extended amount in cents.
pub fn extend_quantity(quantity_milli: i64, unit_price_cents: Cents) -> Cents {
    round_half_up(quantity_milli as i128 * unit_price_cents as i128, 1000)
}

/// Applies a basis-point rate (10000 = 100%) to an amount in cents.
pub fn apply_bp(amount_cents: Cents, bp: i64) -> Cents {
    round_half_up(amount_cents as i128 * bp as i128, 10000)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCalculation {
    pub gross_cents: Cents,
    pub discount_cents: Cents,
    pub net_cents: Cents,
    pub tax_cents: Cents,
    pub line_total_cents: Cents,
}

/// Computes a single document line's totals: gross -> discount -> net -> tax
/// -> line total (inclusive of tax).
pub fn compute_line(
    quantity_milli: i64,
    unit_price_cents: Cents,
    discount_bp: i64,
    tax_rate_bp: i64,
) -> LineCalculation {
    let gross_cents = extend_quantity(quantity_milli, unit_price_cents);
    let discount_cents = apply_bp(gross_cents, discount_bp);
    let net_cents = gross_cents - discount_cents;
    let tax_cents = apply_bp(net_cents, tax_rate_bp);
    let line_total_cents = net_cents + tax_cents;
    LineCalculation {
        gross_cents,
        discount_cents,
        net_cents,
        tax_cents,
        line_total_cents,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DocumentTotals {
    pub subtotal_cents: Cents,
    pub discount_cents: Cents,
    pub tax_cents: Cents,
    pub total_cents: Cents,
}

/// Aggregates a set of already-computed line totals into document-level
/// subtotal/discount/tax/total fields.
pub fn aggregate_lines(lines: &[LineCalculation]) -> DocumentTotals {
    lines.iter().fold(DocumentTotals::default(), |mut acc, l| {
        acc.subtotal_cents += l.gross_cents;
        acc.discount_cents += l.discount_cents;
        acc.tax_cents += l.tax_cents;
        acc.total_cents += l.line_total_cents;
        acc
    })
}

/// Formats cents as a plain major-unit decimal string, e.g. 123456 -> "1234.56".
/// Presentation-only (currency symbol/locale formatting belongs in the UI).
pub fn format_major(cents: Cents) -> String {
    let negative = cents < 0;
    let abs = cents.unsigned_abs();
    let major = abs / 100;
    let minor = abs % 100;
    format!("{}{}.{:02}", if negative { "-" } else { "" }, major, minor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_quantity_without_float_drift() {
        // 2.5 units at $10.00 => $25.00
        assert_eq!(extend_quantity(2500, 1000), 2500);
        // 1 unit (1000 milli) at $9.99 => $9.99
        assert_eq!(extend_quantity(1000, 999), 999);
        // 3 units at $0.10 rounds half up correctly
        assert_eq!(extend_quantity(3000, 10), 30);
    }

    #[test]
    fn applies_basis_point_rates() {
        // 20% tax on $100.00
        assert_eq!(apply_bp(10000, 2000), 2000);
        // 7.5% tax on $19.99, rounds half up: 19.99 * 0.075 = 1.49925 -> 150
        assert_eq!(apply_bp(1999, 750), 150);
    }

    #[test]
    fn computes_line_with_discount_and_tax() {
        // 2 units @ $50.00, 10% discount, 8% tax
        let line = compute_line(2000, 5000, 1000, 800);
        assert_eq!(line.gross_cents, 10000);
        assert_eq!(line.discount_cents, 1000);
        assert_eq!(line.net_cents, 9000);
        assert_eq!(line.tax_cents, 720);
        assert_eq!(line.line_total_cents, 9720);
    }

    #[test]
    fn aggregates_multiple_lines() {
        let l1 = compute_line(1000, 1000, 0, 0);
        let l2 = compute_line(2000, 500, 0, 1000);
        let totals = aggregate_lines(&[l1, l2]);
        assert_eq!(totals.subtotal_cents, 1000 + 1000);
        assert_eq!(totals.tax_cents, 0 + 100);
        assert_eq!(totals.total_cents, 1000 + 1100);
    }

    #[test]
    fn formats_major_units() {
        assert_eq!(format_major(123456), "1234.56");
        assert_eq!(format_major(5), "0.05");
        assert_eq!(format_major(-250), "-2.50");
    }
}
