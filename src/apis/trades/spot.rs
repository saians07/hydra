/*
 * Handle all kinds of spot transaction
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
    async fn private_send_order(
        &self,
        order: OrderRequest,
        order_side: OrderSide,
        order_type: OrderType,
    ) -> Result<OrderResponse, ArgusErr>;
    async fn private_limit_buy_order(
        &self,
        pair: &Pair,
        expected_price: Decimal,
        client_oder_id: Option<String>,
        base_amount: Option<Decimal>,
        quote_amount: Option<Decimal>,
        time_in_force: Option<TimeInForce>,
    ) -> Result<OrderResponse, ArgusErr>;
    async fn private_limit_sell_order(
        &self,
        pair: &Pair,
        expected_price: Decimal,
        client_oder_id: Option<String>,
        base_amount: Option<Decimal>,
        quote_amount: Option<Decimal>,
        time_in_force: Option<TimeInForce>,
    ) -> Result<OrderResponse, ArgusErr>;
    async fn private_limit_scaled_buy_order(
        &self,
        pair: &Pair,
        client_oder_id: Option<String>,
        expected_price: Decimal,
        base_amount: Decimal,
        time_in_force: TimeInForce,
        scales: Vec<f64>,
    ) -> Result<OrderResponse, ArgusErr>;
    async fn private_market_buy_order(
        &self,
        pair: &Pair,
        quote_amount: Decimal,
        client_oder_id: Option<String>,
    ) -> Result<OrderResponse, ArgusErr>;
    async fn private_market_sell_order(
        &self,
        pair: &Pair,
        client_order_id: Option<String>,
        base_amount: Decimal,
    ) -> Result<OrderResponse, ArgusErr>;
}
