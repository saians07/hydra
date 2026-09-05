use argus::errors::ArgusErr;
use async_trait::async_trait;
use polars::frame::DataFrame;
use reqwest::Response;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{self, Display},
    str::FromStr,
};

use crate::core::utils::{RequestType, TimeRange};

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub base_name: Box<str>,
    pub quote_name: Box<str>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub base_name: Box<str>,
    pub quote_name: Box<str>,
    pub order_side: TradeSide, // buy or sell
    pub order_type: Box<str>,  // maker, taker
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

#[derive(Debug, Serialize, Deserialize)]
pub enum TradeSide {
    BUY,
    SELL,
}

#[derive(Debug, Clone)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
    OneHour,
    FourHour,
    OneDay,
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
pub struct Balance {
    pub available: Decimal,
    pub locked: Decimal,
}

impl Display for Balance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "available: {}\n", self.available)?;
        write!(f, "locked: {}\n", self.locked)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CryptoBalance(pub HashMap<String, Balance>);

impl FromStr for CryptoBalance {
    type Err = ArgusErr;
    /// Automatically convert any string balance into hashmap
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower_case = s.to_owned().to_lowercase();

        let mut map = HashMap::new();
        map.insert(lower_case, Balance::default());

        Ok(CryptoBalance(map))
    }
}

impl Display for CryptoBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Wallet Balance: {{\n")?;

        for (token, balance) in &self.0 {
            write!(f, "  {}: \n{},\n", token, balance)?;
        }

        write!(f, "}}")
    }
}

#[async_trait]
pub(crate) trait BaseCex {
    async fn build_payload<T: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: T,
    ) -> Result<Box<str>, ArgusErr>;
    async fn sign_payload(&self, payload: &str) -> Result<Box<str>, ArgusErr>;
    async fn send_request(
        &self,
        url: &str,
        request_type: RequestType,
        body: Option<&str>,
    ) -> Result<Response, ArgusErr>;
    async fn get_recv_window(&self) -> Result<(i64, i64), ArgusErr>;
}

#[async_trait]
pub trait TradeCex {
    async fn private_trade(&self, order: Order) -> Result<TradeResponse, ArgusErr>;
    async fn private_fetch_balance(&self) -> Result<CryptoBalance, ArgusErr>;
    async fn public_fetch_ohlcv(
        &self,
        timeframe: TimeFrame,
        timerange: TimeRange,
        asset: Asset,
    ) -> Result<DataFrame, ArgusErr>;
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

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeFrame::OneMinute => "One Minute",
            TimeFrame::FiveMinutes => "Five Minutes",
            TimeFrame::ThirtyMinutes => "Thirty Minutes",
            TimeFrame::OneHour => "One Hour",
            TimeFrame::FourHour => "Four hour",
            TimeFrame::OneDay => "One Day",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for TimeFrame {
    type Err = ArgusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "One Minute" => Ok(TimeFrame::OneMinute),
            "Five Minutes" => Ok(TimeFrame::FiveMinutes),
            "Thirty Minutes" => Ok(TimeFrame::ThirtyMinutes),
            "One Hour" => Ok(TimeFrame::OneHour),
            "Four Hour" => Ok(TimeFrame::FourHour),
            "One Day" => Ok(TimeFrame::OneDay),
            _ => Err(ArgusErr::operation_ori(format!(
                "Failed to convert unknown string: {}",
                s
            ))),
        }
    }
}

impl TimeFrame {
    pub const ALL_VARIANTS: [TimeFrame; 6] = [
        TimeFrame::OneMinute,
        TimeFrame::FiveMinutes,
        TimeFrame::ThirtyMinutes,
        TimeFrame::OneHour,
        TimeFrame::FourHour,
        TimeFrame::OneDay,
    ];

    /// This is a function fo facilitate market that provides timeframe in minutes period in their API
    pub fn get_period_minute(&self) -> i32 {
        match self {
            TimeFrame::OneMinute => 1,
            TimeFrame::FiveMinutes => 5,
            TimeFrame::ThirtyMinutes => 30,
            TimeFrame::OneHour => 60,
            TimeFrame::FourHour => 240,
            TimeFrame::OneDay => 720,
        }
    }

    /// This is a function to facilitate market that provides their timframe in string
    pub fn get_period_name(&self) -> Box<str> {
        match self {
            TimeFrame::OneMinute => "1m".into(),
            TimeFrame::FiveMinutes => "5m".into(),
            TimeFrame::ThirtyMinutes => "30m".into(),
            TimeFrame::OneHour => "1h".into(),
            TimeFrame::FourHour => "4h".into(),
            TimeFrame::OneDay => "1d".into(),
        }
    }
}
