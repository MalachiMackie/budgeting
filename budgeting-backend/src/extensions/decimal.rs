use rust_decimal::Decimal;

#[allow(clippy::module_name_repetitions)]
pub trait DecimalExt {
    fn approximately_eq(self, other: Decimal, allowance: Decimal) -> bool;
}

impl DecimalExt for Decimal {
    fn approximately_eq(self, other: Decimal, allowance: Decimal) -> bool {
        self.saturating_sub(other).abs() < allowance
    }
}
