/*
 * Handle all kinds of spot transactions.
 *
 * Basically, in transaction, we can do:
 *      - market_order
 *      - limit_order
 *      - scaled_order
 *
 * Those three can be implemented either for buy or sell action
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::core::{
    dtos::{OrderRequest, OrderResponse, Pair},
    enums::{OrderSide, OrderType, TimeInForce},
};

#[async_trait]
pub trait Spot {
    /// Sending request order to the server.
    async fn private_send_order(
        &self,
        order: OrderRequest,
        order_side: OrderSide,
        order_type: OrderType,
    ) -> Result<OrderResponse, ArgusErr>;
    /// Sending a buy maker order
    async fn private_limit_buy_order(
        &self,
        pair: &Pair,
        order_id: Option<i64>,
        expected_price: Decimal,
        base_amount: Option<Decimal>,
        quote_amount: Option<Decimal>,
        time_in_force: TimeInForce,
    ) -> Result<OrderResponse, ArgusErr>;
    /// Sending a buyL taker order
    async fn private_market_buy_order(
        &self,
        pair: &Pair,
        order_id: Option<i64>,
        quote_amount: Option<Decimal>,
    ) -> Result<OrderResponse, ArgusErr>;
    /// Sending scaled order buy (only for limit)
    async fn private_limit_scaled_buy_order(
        &self,
        pair: &Pair,
        order_id: Option<i64>,
        expected_price: Decimal,
        base_amount: Decimal,
        time_in_force: TimeInForce,
        scales: Vec<f64>,
    ) -> Result<OrderResponse, ArgusErr>;
}
