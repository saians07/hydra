use argus::errors::ArgusErr;
use chrono::{DateTime, Days, Local};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::core::enums::{OrderStatus, RequestType};

use super::enums::{OrderSide, OrderType, TimeInForce};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderBook {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Default, Clone)]
pub struct TimeRange {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderRequest {
    pub pair: Pair,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub request_type: RequestType,
    pub client_order_id: Option<String>,
    pub expected_price: Option<Decimal>,    // for limit order
    pub base_amount: Option<Decimal>,       // for market order
    pub quote_amount: Option<Decimal>,      // for limit order
    pub time_in_force: Option<TimeInForce>, // for limit order
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Pair {
    pub base: String,
    pub quote: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderResponse {
    pub pair: Pair,
    pub market_id: Box<str>,
    pub order_id: i64,
    pub order_type: OrderType,
    pub order_side: OrderSide,
    pub trade_id: Vec<i64>,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrderPosition {
    pub symbol: String,
    pub order_id: String,
    pub client_order_id: String,
    pub status: OrderStatus,
}

impl Pair {
    pub fn new(base: String, quote: String) -> Self {
        Pair { base, quote }
    }
    pub fn create_type_1(&self) -> String {
        format!("{}{}", self.base, self.quote)
    }
    pub fn create_type_2(&self) -> String {
        format!("{}_{}", self.base, self.quote)
    }
}

impl TimeRange {
    pub fn new(end_time: Option<DateTime<Local>>, lookback_days: u64) -> Result<Self, ArgusErr> {
        let to_time = match end_time {
            Some(time) => time,
            _ => chrono::Local::now(),
        };

        let to = to_time;
        let from = to_time
            .checked_sub_days(Days::new(lookback_days))
            .unwrap_or(to_time);

        Ok(Self { to, from })
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
