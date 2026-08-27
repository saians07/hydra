use std::{collections::HashMap, fmt::Display, str::FromStr};

use argus::errors::ArgusErr;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Balance {
    pub available: Decimal,
    pub locked: Decimal,
}

impl Display for Balance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "available: {}\n", self.available)?;
        write!(f, "locked: {}\n", self.locked)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BalanceType(pub HashMap<String, Balance>);

impl FromStr for BalanceType {
    type Err = ArgusErr;
    /// Automatically convert any string balance into hashmap
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_owned().to_lowercase();

        let mut map = HashMap::new();
        map.insert(lower_case, Balance::default());

        Ok(BalanceType(map))
    }
}

impl Display for BalanceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wallet Balance: {{\n")?;

        for (token, balance) in &self.0 {
            write!(f, "  {}: \n{},\n", token, balance)?;
        }

        write!(f, "}}")
    }
}

#[async_trait]
pub trait ManageBalance {
    async fn get_balance(&self, asset: &str) -> Decimal;
}
