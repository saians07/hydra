use async_trait::async_trait;
use rust_decimal::Decimal;

pub enum BalanceType {
    IDR,
    USDT,
    TKO,
}

#[async_trait]
pub trait BalanceManager {
    async fn get_balance(&self, balance_type: BalanceType) -> Decimal;
}
