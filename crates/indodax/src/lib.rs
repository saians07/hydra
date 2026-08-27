/*
 * Indodax main library
 * author: Suberlin Sinaga
 */

use std::collections::HashMap;

use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{Days, Local};
use hydra_core::{
    balance::{Balance, BalanceType},
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

    /// Fetch OHLCV data from market server
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
            .map_err(|e| ArgusErr::operation("Could not parse response from Indodax", e))?;

        match json_body.as_array() {
            Some(arr) => {
                let json_string = serde_json::to_string(arr)
                    .map_err(|e| ArgusErr::operation("Failed to convert into json string", e))?;

                let df_ohlcv = JsonReader::new(std::io::Cursor::new(json_string))
                    // .with_json_format(JsonFormat::Json)
                    .finish()
                    .map_err(|e| {
                        ArgusErr::operation("Can not convert Indodax's response to dataframe.", e)
                    })?
                    .lazy()
                    // ensure we have ohlcv instead of OHLCV
                    .with_columns([
                        col("Open")
                            .cast(DataType::Float64)
                            .cast(DataType::Decimal(30, 10))
                            .alias("open"),
                        col("High")
                            .cast(DataType::Float64)
                            .cast(DataType::Decimal(30, 10))
                            .alias("high"),
                        col("Low")
                            .cast(DataType::Float64)
                            .cast(DataType::Decimal(30, 10))
                            .alias("low"),
                        col("Close")
                            .cast(DataType::Float64)
                            .cast(DataType::Decimal(30, 10))
                            .alias("close"),
                        col("Volume")
                            .cast(DataType::Float64)
                            .cast(DataType::Decimal(30, 10))
                            .alias("volume"),
                    ])
                    .with_column((col("Time") * lit(1000)).alias("time"))
                    .drop(by_name(
                        ["Open", "High", "Low", "Close", "Volume", "Time"],
                        true,
                        false,
                    ))
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
    pub async fn private_get_info(&self) -> Result<BalanceType, ArgusErr> {
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

        let account_info = result.json::<IndodaxResponse>().await.map_err(|e| {
            ArgusErr::operation(
                "Failed to convert Indodax's account info ro Indodax general response.",
                e,
            )
        })?;

        let balance_value = account_info
            .clone()
            .returned_data
            .unwrap()
            .get("balance")
            .ok_or_else(|| ArgusErr::operation_ori("Failed to extract Indodax balance"))?
            .to_owned();

        let locked_value = account_info
            .returned_data
            .unwrap()
            .get("balance_hold")
            .ok_or_else(|| ArgusErr::operation_ori("Failed to extract Indodax balance"))?
            .as_object()
            .ok_or_else(|| {
                ArgusErr::operation_ori(
                    "Failed to convert Indodax locked balance &Value to an object",
                )
            })?
            .to_owned();

        let balance = balance_value
            .as_object()
            .ok_or_else(|| {
                ArgusErr::operation_ori("Failed to convert Indodax balance &Value to an object")
            })?
            .into_iter()
            .map(|(key, value)| {
                let mut final_balance = Balance::default();
                let balance = serde_json::from_value(value.clone()).map_err(|e| {
                    ArgusErr::operation(
                        "Failed to convert Indodax json balance Value to be Balance",
                        e,
                    )
                })?;
                final_balance.available = balance;

                if let Some(locked_val) = locked_value.get(key) {
                    let locked_bal = serde_json::from_value(locked_val.clone()).map_err(|e| {
                        ArgusErr::operation(
                            "Failed to convert Indodax json locked Value to be Balance",
                            e,
                        )
                    })?;
                    final_balance.locked = locked_bal;
                }

                Ok((key.clone(), final_balance))
            })
            .collect::<Result<HashMap<String, Balance>, ArgusErr>>()?;

        Ok(BalanceType(balance))
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use reqwest::Client;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::prelude::*;
    use serde::Serialize;

    // Ensure that the url will come from our tokio server mock
    fn create_test_client(public_url: &str, private_url: &str) -> Indodax {
        Indodax {
            id: Box::from("indodax"),
            name: Box::from("INDODAX"),
            api_key: SecretString::new("dummy_api_key".into()),
            secret_key: SecretString::new("dummy_secret_key".into()),
            private_api_url: private_url.into(),
            public_api_url: public_url.into(),
            private_ws_url: "".into(),
            public_ws_url: "".into(),
            client: Client::new(),
        }
    }

    #[derive(Serialize)]
    struct DummyPayload {
        method: String,
        pair: String,
    }

    #[tokio::test]
    async fn test_private_get_info() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"success": 1, "return": {"balance": {"btc": 1}, "balance_hold": {"idr": 0}}}"#,
            )
            .create_async()
            .await;

        let indodax = create_test_client(&mock_url, &mock_url);

        let response = indodax.private_get_info().await;

        // assert!(response.is_ok());
        let res = response.unwrap();
        assert_eq!(
            res.0.get("btc").unwrap_or(&Balance::default()).available,
            Decimal::from_i32(1).unwrap_or_default()
        );
        assert_eq!(
            res.0.get("idr").unwrap_or(&Balance::default()).locked,
            Decimal::from_i32(0).unwrap_or_default()
        );
        mock.assert_async().await; // Ensure the mock is trully called
    }

    #[tokio::test]
    async fn test_create_time_range() {
        // test if the difference between the from and to of the TimeRange
        // is equal to and greater than 7 days (604800) and less than two second
        // from 604800
        let indodax = create_test_client("http://dummy", "http://dummy");

        let lookback_days = 7;
        let time_range = indodax.create_time_range(lookback_days).await.unwrap();

        let from_ts: i64 = time_range.from.parse().unwrap();
        let to_ts: i64 = time_range.to.parse().unwrap();

        // With lookback 7 days = 7 * 24 * 60 * 60 = 604800 seconds
        // Gives 1-2 seconds tolerance due to execution delay
        let diff = to_ts - from_ts;
        assert!(
            diff >= 604800 && diff <= 604802,
            "Difference not as expected: {}",
            diff
        );
    }

    #[tokio::test]
    async fn test_build_payload() {
        let indodax = create_test_client("http://dummy", "http://dummy");

        let payload = DummyPayload {
            method: "trade".to_string(),
            pair: "btc_idr".to_string(),
        };

        let result = indodax.build_payload(payload).await.unwrap();

        // Ensure the serde_qs can successfully format the struct to be form-urlencoded
        assert_eq!(result.as_ref(), "method=trade&pair=btc_idr");
    }

    #[tokio::test]
    async fn test_sign_payload() {
        let body = "halo";
        let mut indodax = create_test_client("http://dummy", "http://dummy");
        indodax.secret_key = "123".into();

        assert_eq!(
            indodax.sign_payload(body).await.unwrap(),
            Box::from(
                "b63f4bbf2898cc5a279dd3cd96722ca0977e545ad3bd09a63b63fdc361cf776eda1dd97fdb444a0c4c230333897302aa79ea6b029c07e58763f0b62f487e1da6"
            )
        )
    }

    #[tokio::test]
    async fn test_send_request_get_ok() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        let mock = server.mock("GET", "/test-error").create_async().await;

        let indodax = create_test_client(&mock_url, &mock_url);
        let url = format!("{}/test-error", mock_url);

        let response = indodax
            .send_request(&url, RequestType::GET, &indodax.client, None)
            .await;

        assert!(response.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_send_request_post_ok() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        let mock = server.mock("POST", "/test-error").create_async().await;

        let indodax = create_test_client(&mock_url, &mock_url);
        let url = format!("{}/test-error", mock_url);

        let response = indodax
            .send_request(&url, RequestType::POST, &indodax.client, None)
            .await;

        assert!(response.is_ok());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_send_request_get_error_400() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("GET", "/test-error")
            .with_status(400)
            .create_async()
            .await;

        let indodax = create_test_client(&mock_url, &mock_url);
        let url = format!("{}/test-error", mock_url);

        let response = indodax
            .send_request(&url, RequestType::GET, &indodax.client, None)
            .await;

        assert!(response.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_send_request_post_error_401() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        let mock = server
            .mock("POST", "/test-error")
            .with_status(401)
            .create_async()
            .await;

        let indodax = create_test_client(&mock_url, &mock_url);
        let url = format!("{}/test-error", mock_url);

        let response = indodax
            .send_request(&url, RequestType::POST, &indodax.client, None)
            .await;

        assert!(response.is_err());
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_recv_window() {
        // testing if recv_window is exactly now - 4000
        let indodax = create_test_client("http://dummy", "http://dummy");
        let (now, recv_window) = indodax.get_recv_window().await.unwrap();

        assert_eq!(recv_window, now - 4000);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_public_fetch_ohlcv_success() {
        let mut server = Server::new_async().await;
        let mock_url = server.url();

        // 1. Siapkan Mock Response JSON
        let mock_body = r#"[
                {"Time": 1620000000, "Open": 50000.0, "High": 51000.0, "Low": 49000.0, "Close": 50500.0, "Volume": "0.0002"},
                {"Time": 1620003600, "Open": 50500.0, "High": 52000.0, "Low": 50000.0, "Close": 51500.0, "Volume": "2.0"}
            ]"#;

        let mock = server
            .mock(
                "GET",
                mockito::Matcher::Regex(r"^/tradingview/history_v2".into()),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_body)
            .create_async()
            .await;

        let indodax = create_test_client(&mock_url, &mock_url);

        // 2. Siapkan parameter TimeFrame dan TimeRange sesuai Struct asli
        let time_frame = TimeFrame::OneHour;
        let time_range = TimeRange {
            from: Box::from("1620000000"),
            to: Box::from("1620003600"),
        };

        // 3. Eksekusi Fungsi
        let df_result = indodax.public_fetch_ohlcv(time_frame, time_range).await;

        // 4. Verifikasi
        assert!(
            df_result.is_ok(),
            "Gagal memproses DataFrame: {:?}",
            df_result.err()
        );
        let df = df_result.unwrap();

        assert_eq!(df.height(), 2);

        let data_decimal: [Decimal; 2] = [Decimal::new(2, 4), Decimal::new(2000, 3)];

        let float_vec: Vec<f64> = data_decimal
            .iter()
            .map(|d| d.to_f64().unwrap_or(0.0))
            .collect();

        let col = Column::new("volume".into(), float_vec)
            .cast(&DataType::Decimal(30 as usize, 10 as usize))
            .unwrap();

        let column_names = df.get_column_names();
        assert!(column_names.iter().any(|name| name.as_str() == "open"));
        assert!(column_names.iter().any(|name| name.as_str() == "time"));
        assert_eq!(df.column("volume").unwrap(), &col);

        mock.assert_async().await;
    }
}
