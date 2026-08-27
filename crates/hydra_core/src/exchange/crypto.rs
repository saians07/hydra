use std::str::FromStr;

use argus::errors::ArgusErr;
use async_trait::async_trait;
use polars::frame::DataFrame;
use reqwest::{Client, Response};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{timeframe::TimeFrame, utils::RequestType};

#[async_trait]
pub trait CexMarket {
    async fn build_payload<T: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: T,
    ) -> Result<Box<str>, ArgusErr>;
    async fn sign_payload(&self, payload: &str) -> Result<Box<str>, ArgusErr>;
    async fn send_request(
        &self,
        url: &str,
        request_type: RequestType,
        client: &Client,
        body: Option<&str>,
    ) -> Result<Response, ArgusErr>;
    async fn get_recv_window(&self) -> Result<(i64, i64), ArgusErr>;
    async fn private_trade(&self, order: Order) -> Result<TradeResponse, ArgusErr>;
    async fn public_fetch_ohlcv(
        &self,
        timeframe: TimeFrame,
        timerange: TimeRange,
        asset: &Asset,
    ) -> Result<DataFrame, ArgusErr>;
}

// this struct defines what attributes will be available for an assets.
// this will extract information from database
pub struct Asset {
    pub base_name: Box<str>,
    pub quote_name: Box<str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Side {
    BUY,
    SELL,
}

#[derive(Debug, Default)]
pub struct TimeRange {
    pub from: Box<str>,
    pub to: Box<str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub base_name: Box<str>,
    pub quote_name: Box<str>,
    pub order_side: Side,     // buy or sell
    pub order_type: Box<str>, // maker, taker
    pub base_qty: Option<Decimal>,
    pub quote_qyt: Option<Decimal>,
    pub price: Option<Decimal>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TradeResponse {
    pub fee_cost: Decimal,
    pub tax_cost: Option<Decimal>,
    pub base_amount: Option<Decimal>,
    pub quote_amount: Option<Decimal>,
}

impl TradeResponse {
    /// Convert any serde_json::Value into Decimal
    pub async fn convert_to_decimal(value: &Value) -> Result<Decimal, ArgusErr> {
        let decimal_value = match value {
            Value::String(s) => Decimal::from_str(s)
                .map_err(|e| ArgusErr::operation("Failed to convert data to Decimal", e))?,
            Value::Number(n) => Decimal::from_str(&n.to_string())
                .map_err(|e| ArgusErr::operation("Failed to convert data to Decimal", e))?,
            _ => {
                return Err(ArgusErr::operation_ori(
                    "Can not convert values outside string and number into Decimal.",
                ));
            }
        };

        Ok(decimal_value)
    }
}
