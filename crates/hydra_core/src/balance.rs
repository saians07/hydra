use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Balance {
    pub available: Decimal,
    pub locked: Decimal,
    pub freeze: Decimal,
}

#[async_trait]
pub trait ManageBalance {
    async fn get_balance(&self, asset: &str) -> Decimal;
}
