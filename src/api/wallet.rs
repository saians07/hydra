/*
 * Manage all balances related to the wallet
 */

use std::{collections::HashMap, fmt::Display, sync::Arc};

use argus::errors::ArgusErr;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Balance {
    pub available: Decimal,
    pub locked: Decimal,
}

#[derive(Debug, Clone)]
pub struct Balances(pub Arc<RwLock<HashMap<Box<str>, Balance>>>);

impl Balances {
    pub async fn new() -> Self {
        Balances(Arc::new(RwLock::new(HashMap::new())))
    }
}

#[async_trait]
pub trait Wallet {
    async fn private_fetch_balances(&self) -> Result<Balances, ArgusErr>;
}

impl Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Balance:\n {:#?}", self)
    }
}
