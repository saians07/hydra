use std::collections::HashMap;

use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{Days, Local};
use hydra_core::{
    exchange::crypto::{CexMarket, Order, Side, TimeRange, TradeResponse},
    timeframe::TimeFrame,
    utils::{RequestType, hmac_512},
};
use polars::prelude::*;
use reqwest::{Client, Response, header::HeaderValue};
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_qs::to_string;

#[derive(Debug, Clone)]
pub struct Indodax {
    pub id: Box<str>,
    pub name: Box<str>,
    api_key: SecretString,
    secret_key: SecretString,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Box<str>,
    pub public_ws_url: Box<str>,
    pub client: Client,
}

// Indodax has no side member, it only has type for side and order_type
#[derive(Debug, Serialize)]
struct IndodaxOrder {
    pub method: Box<str>,
    pub timestamp: i64,
    pub recv_window: i64,
    pub pair: Box<str>,
    #[serde(rename = "type")]
    pub side: Box<str>,
    order_type: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Decimal>,
    // we need change this to the specific quote name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_qty: Option<Decimal>,
}

// this will capture all kinds of indodax response
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct IndodaxResponse {
    #[serde(rename = "return")]
    pub returned_data: Option<HashMap<String, Value>>,
    pub success: Option<i32>,
    pub error: Option<Box<str>>,
    pub message: Option<Box<str>>,
    pub tickers: Option<HashMap<String, Value>>,
}

impl Indodax {
    pub fn new(api_key: SecretString, secret_key: SecretString, client: Client) -> Self {
        Self {
            id: Box::from("indodax"),
            name: Box::from("INDODAX"),
            api_key,
            secret_key,
            private_api_url: "https://indodax.com/tapi".into(),
            public_api_url: "https://indodax.com".into(),
            private_ws_url: "".into(),
            public_ws_url: "".into(),
            client,
        }
    }
}

#[async_trait]
impl CexMarket for Indodax {
    /// Serialize any struct to build the payload data.
    async fn build_payload<P: serde::Serialize + Send + Sync + 'static>(
        &self,
        payload: P,
    ) -> Result<Box<str>, ArgusErr> {
        match serde_qs::to_string(&payload) {
            Ok(payload) => return Ok(Box::from(payload)),
            Err(e) => return Err(ArgusErr::operation("Failed to build payload:", e)),
        }
    }

    async fn sign_payload(&self, payload: &str) -> Result<Box<str>, ArgusErr> {
        let result = hmac_512(payload, self.secret_key.expose_secret())?;

        Ok(Box::from(result))
    }

    async fn send_request(
        &self,
        url: &str,
        request_type: RequestType,
        client: &Client,
        body: Option<&str>,
    ) -> Result<Response, ArgusErr> {
        match request_type {
            RequestType::GET => {
                let result = client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| ArgusErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code:", e))?;

                Ok(result)
            }
            RequestType::POST => {
                let body = body.unwrap_or("");
                let signed_body = self.sign_payload(body).await?;
                let result = client
                    .post(url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header(
                        "Key",
                        HeaderValue::from_str(&self.api_key.expose_secret()).unwrap(),
                    )
                    .header("Sign", HeaderValue::from_str(&signed_body).unwrap())
                    .body(body.to_owned())
                    .send()
                    .await
                    .map_err(|e| ArgusErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code:", e))?;

                Ok(result)
            }
        }
    }

    async fn get_recv_window(&self) -> Result<(i64, i64), ArgusErr> {
        let now = chrono::Local::now().timestamp_millis();

        // this will be deduced with how long the request is valid for
        let recv_window = now - 4000;

        Ok((now, recv_window))
    }

    async fn private_trade(&self, order: Order) -> Result<TradeResponse, ArgusErr> {
        let (now, recv_window) = self.get_recv_window().await?;
        let side: Box<str> = match order.order_side {
            Side::BUY => "buy".into(),
            Side::SELL => "sell".into(),
        };
        // convert general order to specific order market
        let market_order = IndodaxOrder {
            method: "trade".into(),
            timestamp: now,
            recv_window,
            pair: format!(
                "{}_{}",
                order.base_name.to_lowercase(),
                order.quote_name.to_lowercase()
            )
            .into(),
            side,
            order_type: order.order_type,
            price: order.price,
            quote_qty: order.quote_qyt,
            base_qty: order.base_qty,
        };
        let body = self
            .build_payload(market_order)
            .await?
            .replace(
                "quote_qty=",
                format!("{}=", order.quote_name.to_lowercase()).as_str(),
            )
            .replace(
                "base_qty=",
                format!("{}=", order.base_name.to_lowercase()).as_str(),
            );

        let result = self
            .send_request(
                &self.private_api_url,
                RequestType::POST,
                &self.client,
                Some(&body),
            )
            .await?;

        // check the HTTP response from the server
        result
            .error_for_status_ref()
            .map_err(|e| ArgusErr::operation("Failed to send request to Indodax", e))?;

        // convert the response to be a text and then check if it is successfully converted
        let raw_text = result.text().await.map_err(|e| {
            ArgusErr::operation("Failed to convert Indodax response to a raw text", e)
        })?;

        let data: IndodaxResponse = serde_json::from_str(&raw_text)
            .map_err(|e| ArgusErr::operation("Failed to convert response to IndodaxResponse", e))?;

        let returned_data = data
            .returned_data
            .as_ref()
            .ok_or_else(|| ArgusErr::operation_ori("Failed to extract data from response"))?;

        let base_key = match order.order_side {
            Side::BUY => "receive",
            Side::SELL => "sold",
        };

        let quote_key = match order.order_side {
            Side::BUY => "spend",
            Side::SELL => "receive",
        };

        let quote_name: Box<str> = match order.quote_name.as_ref() {
            "idr" => Box::from("rp"),
            _ => Box::from(order.quote_name.to_lowercase()),
        };

        let response = TradeResponse {
            fee_cost: TradeResponse::convert_to_decimal(
                returned_data
                    .get("fee")
                    .ok_or_else(|| ArgusErr::operation_ori("Failed to extract fee amount."))?,
            )
            .await?,
            tax_cost: None,
            base_amount: Some(
                TradeResponse::convert_to_decimal(
                    returned_data
                        .get(format!("{}_{}", base_key, order.base_name.to_lowercase()).as_str())
                        .ok_or_else(|| {
                            ArgusErr::operation_ori("Failed to extract base currency data")
                        })?,
                )
                .await?,
            ),
            quote_amount: Some(
                TradeResponse::convert_to_decimal(
                    returned_data
                        .get(format!("{}_{}", quote_key, quote_name).as_str())
                        .ok_or_else(|| {
                            ArgusErr::operation_ori("Failed to extract quote currencty data.")
                        })?,
                )
                .await?,
            ),
        };

        Ok(response)
    }

    async fn public_fetch_ohlcv(
        &self,
        timeframe: TimeFrame,
        timerange: TimeRange,
    ) -> Result<DataFrame, ArgusErr> {
        let base_url = format!("{}/tradingview/history_v2?", self.public_api_url);
        let url = format!(
            "{}from={}&to={}&tf={}",
            base_url,
            timerange.from,
            timerange.to,
            timeframe.get_period_minutes()
        );
        let resp = self
            .send_request(&url, RequestType::GET, &self.client, None)
            .await?;

        let json_body: Value = resp
            .json()
            .await
            .map_err(|e| ArgusErr::operation("Can not parse response from Indodax: ", e))?;

        match json_body.as_array() {
            Some(arr) => {
                let json_string =
                    to_string(arr).map_err(|e| ArgusErr::operation("Failed to ", e))?;
                let df_ohlcv = JsonReader::new(std::io::Cursor::new(json_string))
                    .finish()
                    .map_err(|e| {
                        ArgusErr::operation("Can not convert Indodax's response to dataframe.", e)
                    })?
                    .lazy()
                    // ensure we have ohlcv instead of OHLCV
                    .with_columns([
                        col("Open").cast(DataType::Decimal(30, 10)).alias("open"),
                        col("High").cast(DataType::Decimal(30, 10)).alias("high"),
                        col("Low").cast(DataType::Decimal(30, 10)).alias("low"),
                        col("Close").cast(DataType::Decimal(30, 10)).alias("close"),
                        col("Volume")
                            .cast(DataType::Decimal(30, 10))
                            .alias("volume"),
                    ])
                    .with_column((col("Time") * lit(1000)).alias("time"))
                    .drop(by_name(["Open", "High"], true, false))
                    .collect()
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert dataframe to be ohlcv", e)
                    })?;
                Ok(df_ohlcv)
            }
            None => Err(ArgusErr::operation_ori("No content found in the response.")),
        }
        // Ok(DataFrame::default())
    }
}

// this is the specific implementation of Indodax endpoint.
impl Indodax {
    /// Private getInfo endpoint of Indondax
    pub async fn private_get_info(&self) -> Result<Response, ArgusErr> {
        let (now, recv_window) = self.get_recv_window().await?;
        let body = format!(
            "method=getInfo&timestamp={:?}&recvWindow={:?}",
            now, recv_window
        );

        let result = self
            .send_request(
                &self.private_api_url,
                RequestType::POST,
                &self.client,
                Some(&body),
            )
            .await?;

        Ok(result)
    }

    pub async fn create_time_range(&self, lookback_days: u64) -> Result<TimeRange, ArgusErr> {
        let mut a = TimeRange::default();
        let current_time = Local::now();
        a.to = current_time.timestamp().to_string().into();
        a.from = current_time
            .checked_sub_days(Days::new(lookback_days))
            .unwrap_or(current_time)
            .timestamp()
            .to_string()
            .into();
        Ok(a)
    }
}
