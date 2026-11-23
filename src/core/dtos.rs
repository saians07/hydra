use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::enums::{OrderSide, OrderType, TimeInForce};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderBook {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OrderRequest {
    pub pair: Pair,
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

impl Pair {
    pub fn new(base: String, quote: String) -> Self {
        Pair { base, quote }
    }
}
