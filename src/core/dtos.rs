use argus::errors::ArgusErr;
use chrono::{Days, prelude::*};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::enums::{OrderSide, OrderType, TimeInForce};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderBook {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderRequest {
    pub pair: Pair,
    pub client_order_id: Option<i64>,
    pub expected_price: Option<Decimal>,    // for limit order
    pub base_amount: Option<Decimal>,       // for market order
    pub quote_amount: Option<Decimal>,      // for limit order
    pub time_in_force: Option<TimeInForce>, // for limit order
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub pair: Pair,
    pub market_id: Box<str>,
    pub order_id: i64,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub trade_id: Vec<i64>,
    pub client_order_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Pair {
    pub base: String,
    pub quote: String,
}

/// Store how long a time range will be applied. Useful to grab some data.
/// This will be very useful when we needs to grab some data.
/// We simply create the TimeRange that will be guaranteed in
/// chrono local DateTime.
#[derive(Debug, Default, Clone)]
pub struct TimeRange {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

impl Pair {
    pub fn new(base: String, quote: String) -> Self {
        Pair { base, quote }
    }
}

impl TimeRange {
    pub fn new(end_time: Option<DateTime<Local>>, lookback_days: u64) -> Result<Self, ArgusErr> {
        let to_time = match end_time {
            Some(time) => time,
            _ => chrono::Local::now(),
        };

        let mut a = Self::default();

        a.to = to_time;
        a.from = to_time
            .checked_sub_days(Days::new(lookback_days))
            .unwrap_or(to_time);

        Ok(a)
    }

    pub fn to_timestamp(&self) -> Result<(i64, i64), ArgusErr> {
        let to: i64 = self.to.timestamp();
        let from: i64 = self.from.timestamp();

        Ok((to, from))
    }

    pub fn to_str(&self) -> Result<(Box<str>, Box<str>), ArgusErr> {
        let (to, from) = self.to_timestamp()?;

        Ok((to.to_string().into(), from.to_string().into()))
    }
}
