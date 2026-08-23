use std::collections::HashMap;

use argus::errors::CustomErr;
use async_trait::async_trait;
use hydra_core::{
    market::crypto::{CexMarket, Order, Side, TradeResponse},
    utils::{RequestType, hmac_512},
};
use reqwest::{Client, Response, header::HeaderValue};
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    ) -> Result<Box<str>, CustomErr> {
        match serde_qs::to_string(&payload) {
            Ok(payload) => return Ok(Box::from(payload)),
            Err(e) => return Err(CustomErr::operation("Failed to build payload:", e)),
        }
    }

    async fn sign_payload(&self, payload: &str) -> Result<Box<str>, CustomErr> {
        let result = hmac_512(payload, self.secret_key.expose_secret())?;

        Ok(Box::from(result))
    }

    async fn send_request(
        &self,
        body: &str,
        url: &str,
        request_type: RequestType,
        client: &Client,
    ) -> Result<Response, CustomErr> {
        match request_type {
            RequestType::GET => {
                let result = client
                    .get(url)
                    .send()
                    .await
                    .map_err(|e| CustomErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| CustomErr::operation("Error with status code:", e))?;

                Ok(result)
            }
            RequestType::POST => {
                let signed_body = self.sign_payload(body).await?;
                let result = client
                    .post(url)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .header(
                        "Key",
                        HeaderValue::from_str(&self.api_key.expose_secret()).unwrap(),
                    )
                    .header("Sign", HeaderValue::from_str(&signed_body).unwrap())
                    .send()
                    .await
                    .map_err(|e| CustomErr::operation("Failed to request data from Indodax", e))?
                    .error_for_status()
                    .map_err(|e| CustomErr::operation("Error with status code:", e))?;

                Ok(result)
            }
        }
    }

    async fn get_recv_window(&self) -> Result<(i64, i64), CustomErr> {
        let now = chrono::Local::now().timestamp_millis();

        // this will be deduced with how long the request is valid for
        let recv_window = now - 4000;

        Ok((now, recv_window))
    }

    async fn private_trade(&self, order: Order) -> Result<TradeResponse, CustomErr> {
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
                &body,
                &self.private_api_url,
                RequestType::POST,
                &self.client,
            )
            .await?;

        // check the HTTP response from the server
        result
            .error_for_status_ref()
            .map_err(|e| CustomErr::operation("Failed to send request to Indodax", e))?;

        // convert the response to be a text and then check if it is successfully converted
        let raw_text = result.text().await.map_err(|e| {
            CustomErr::operation("Failed to convert Indodax response to a raw text", e)
        })?;

        let data: IndodaxResponse = serde_json::from_str(&raw_text).map_err(|e| {
            CustomErr::operation("Failed to convert response to IndodaxResponse", e)
        })?;

        let returned_data = data
            .returned_data
            .as_ref()
            .ok_or_else(|| CustomErr::operation_ori("Failed to extract data from response"))?;

        let base_key = match order.order_side {
            Side::BUY => "receive",
            Side::SELL => "sold",
        };

        let quote_key = match order.order_side {
            Side::BUY => "spend",
            Side::SELL => "receive",
        };

        let response = TradeResponse {
            fee_cost: TradeResponse::convert_to_decimal(
                returned_data
                    .get("fee")
                    .ok_or_else(|| CustomErr::operation_ori("Failed to extract fee amount."))?,
            )
            .await?,
            tax_cost: None,
            base_amount: Some(
                TradeResponse::convert_to_decimal(
                    returned_data
                        .get(format!("{}_{}", base_key, order.base_name.to_lowercase()).as_str())
                        .ok_or_else(|| {
                            CustomErr::operation_ori("Failed to extract base currency data")
                        })?,
                )
                .await?,
            ),
            quote_amount: Some(
                TradeResponse::convert_to_decimal(
                    returned_data
                        .get(format!("{}_{}", quote_key, order.quote_name.to_lowercase()).as_str())
                        .ok_or_else(|| {
                            CustomErr::operation_ori("Failed to extract quote currencty data.")
                        })?,
                )
                .await?,
            ),
        };

        Ok(response)
    }
}

// this is the specific implementation of Indodax endpoint.
impl Indodax {
    /// Private getInfo endpoint of Indondax
    pub async fn private_get_info(&self) -> Result<Response, CustomErr> {
        let (now, recv_window) = self.get_recv_window().await?;
        let body = format!(
            "method=getInfo&timestamp={:?}&recvWindow={:?}",
            now, recv_window
        );

        let result = self
            .send_request(
                &body,
                &self.private_api_url,
                RequestType::POST,
                &self.client,
            )
            .await?;

        Ok(result)
    }
}
