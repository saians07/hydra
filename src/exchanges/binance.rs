use std::{fmt::Display, str::FromStr};

use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use polars::{df, frame::DataFrame};
use reqwest::{Client, header::HeaderMap};
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_with::{DisplayFromStr, serde_as};

use crate::{
    apis::{
        account::Account,
        data::fetch::DataFetch,
        exchange::Exchange,
        request::Request,
        wallet::{Balance, Balances, Wallet},
    },
    core::{
        dtos::{OrderBook, Pair, TimeRange},
        enums::{Hasher, RequestType, TimeFrame},
    },
};

#[derive(Debug, Clone)]
pub struct Binance {
    api_key: SecretString,
    secret_key: SecretString,
    client: Client,
    pub id: Box<str>,
    pub name: Box<str>,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Option<Box<str>>,
    pub public_ws_url: Option<Box<str>>,
    pub api_url_testing: Option<Box<str>>,
}

#[async_trait]
impl Account for Binance {
    async fn private_fetch_account_information<T: DeserializeOwned>(&self) -> Result<T, ArgusErr> {
        let query = format!("timestamp={}", chrono::Local::now().timestamp_millis());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );

        // account needs query to be signed
        let signed_query = self.sign_request(&query).await?;

        let endpoint = format!(
            "{}{}{}&signature={}",
            self.private_api_url, "/api/v3/account?", query, signed_query
        );

        self.send_request::<T>(&endpoint, RequestType::GET, None, headers)
            .await
    }
}

#[async_trait]
impl DataFetch for Binance {
    async fn public_fetch_order_book(&self, pair: &Pair) -> Result<OrderBook, ArgusErr> {
        let symbol = format!("symbol={}{}", pair.base, pair.quote);
        let endpoint = format!("{}{}?{}", self.public_api_url, "/api/v3/depth", symbol);
        let headers = HeaderMap::new();

        self.send_request::<OrderBook>(&endpoint, RequestType::GET, None, headers)
            .await
    }

    async fn public_fetch_ohlcv(
        &self,
        time_frame: TimeFrame,
        time_range: TimeRange,
        pair: &Pair,
    ) -> Result<DataFrame, ArgusErr> {
        let query = format!(
            "symbol={}{}&interval={}&startTime={}&endTime={}",
            pair.base,
            pair.quote,
            time_frame.get_period_name(),
            time_range.from.with_timezone(&Utc).timestamp() * 1000,
            time_range.to.with_timezone(&Utc).timestamp() * 1000
        );
        let endpoint = format!("{}{}?{}", self.public_api_url, "/api/v3/klines", query);
        let headers = HeaderMap::new();

        let res = self
            .send_request::<Vec<BNBOhlcv>>(&endpoint, RequestType::GET, None, headers)
            .await?;

        let df = df!(
            "open" => res.iter().map(|x| x.open).collect::<Vec<_>>(),
            "high" => res.iter().map(|x| x.high).collect::<Vec<_>>(),
            "low" => res.iter().map(|x| x.low).collect::<Vec<_>>(),
            "close" => res.iter().map(|x| x.closing).collect::<Vec<_>>(),
            "base_volume" => res.iter().map(|x| x.base_volume).collect::<Vec<_>>(),
            "quote_volume" => res.iter().map(|x| x.quote_volume).collect::<Vec<_>>(),
            "open_time" => res.iter().map(|x| {x.open_time}).collect::<Vec<_>>(),
            "close_time" => res.iter().map(|x| {x.close_time}).collect::<Vec<_>>(),
            "trades_count" => res.iter().map(|x| x.trades_count).collect::<Vec<_>>(),
            "taker_buy_base_volume" => res.iter().map(|x| x.taker_base_volume).collect::<Vec<_>>(),
            "taker_buy_quote_volume" => res.iter().map(|x| x.taker_quote_volume).collect::<Vec<_>>(),
        )
        .map_err(|e| ArgusErr::operation("Failed to convert OHLCV data to data frame", e))?;

        Ok(df)
    }
}

#[async_trait]
impl Exchange for Binance {
    async fn public_get_server_time(&self) -> Result<DateTime<Local>, ArgusErr> {
        let endpoint = format!("{}{}", self.public_api_url, "/api/v3/time");
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let res = self
            .send_request::<serde_json::Value>(&endpoint, RequestType::GET, None, headers)
            .await?;

        let milis = res
            .get("serverTime")
            .unwrap_or_default()
            .to_string()
            .parse::<i64>()
            .map_err(|e| ArgusErr::operation("Failed to parse server time to timestamp.", e))?;

        let date_time = chrono::DateTime::from_timestamp_millis(milis)
            .unwrap()
            .with_timezone(&Local);

        println!("Result: {:#?}", date_time);

        Ok(date_time)
    }
}

#[async_trait]
impl Request for Binance {
    async fn sign_request(&self, payload: &str) -> Result<Box<str>, ArgusErr> {
        let hasher = Hasher::Sha256;

        let signed_body = hasher.hash(payload, self.secret_key.expose_secret())?;

        Ok(Box::from(signed_body))
    }

    async fn send_request<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        request_type: RequestType,
        body: Option<&str>,
        headers: HeaderMap,
    ) -> Result<T, ArgusErr> {
        match request_type {
            RequestType::GET => {
                let request = self
                    .client
                    .get(endpoint)
                    .header("Accept", "application/json")
                    .headers(headers);

                request
                    .send()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to send request to the Binance server!", e)
                    })?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code", e))?
                    .json()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert Binance's response to json", e)
                    })
            }
            RequestType::POST => {
                let request = self
                    .client
                    .post(endpoint)
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .headers(headers)
                    .body(body.unwrap().to_owned());

                request
                    .send()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to post request to Binance server!", e)
                    })?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code", e))?
                    .json()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert Binance's response to json", e)
                    })
            }
            RequestType::DELETE => Err(ArgusErr::operation_ori(
                "DELETE request type is not yet supported!",
            )),
            RequestType::PATCH => Err(ArgusErr::operation_ori(
                "PATCH request type is not yet supported!",
            )),
            RequestType::PUT => Err(ArgusErr::operation_ori(
                "PUT request type is not yet supported!",
            )),
        }
    }
}

#[async_trait]
impl Wallet for Binance {
    async fn private_fetch_balances(&self) -> Result<Balances, ArgusErr> {
        let account_info = self
            .private_fetch_account_information::<BNBAccountInfo>()
            .await?;

        account_info.get_balance().await
    }
}

impl Display for Binance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The market is: {}", self.name)
    }
}

impl Binance {
    pub fn new(api_key: SecretString, secret_key: SecretString) -> Self {
        Self {
            id: "binance".into(),
            name: "Binance".into(),
            api_key,
            secret_key,
            client: Client::new(),
            private_api_url: "https://api.binance.com".into(),
            public_api_url: "https://api.binance.com".into(),
            private_ws_url: None,
            public_ws_url: None,
            api_url_testing: None,
        }
    }
}

/*
 * LIST OF Binance's specific struct
 */
#[derive(Debug, Serialize, Deserialize)]
struct BNBAccountInfo {
    #[serde(rename = "makerCommission")]
    maker_commission: f64,
    #[serde(rename = "takerCommission")]
    taker_commission: f64,
    pub balances: Vec<BNBBalance>,
    #[serde(rename = "accountType")]
    account_type: String,
    permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BNBBalance {
    asset: Box<str>,
    free: Box<str>,
    locked: Box<str>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
struct BNBOhlcv {
    open_time: i64,
    #[serde_as(as = "DisplayFromStr")]
    open: f64,
    #[serde_as(as = "DisplayFromStr")]
    high: f64,
    #[serde_as(as = "DisplayFromStr")]
    low: f64,
    #[serde_as(as = "DisplayFromStr")]
    closing: f64,
    #[serde_as(as = "DisplayFromStr")]
    base_volume: f64,
    close_time: i64,
    #[serde_as(as = "DisplayFromStr")]
    quote_volume: f64,
    trades_count: i64,
    #[serde_as(as = "DisplayFromStr")]
    taker_base_volume: f64,
    #[serde_as(as = "DisplayFromStr")]
    taker_quote_volume: f64,
    #[serde_as(as = "DisplayFromStr")]
    ignore: String,
}

impl BNBAccountInfo {
    async fn get_balance(&self) -> Result<Balances, ArgusErr> {
        let balances = Balances::new().await;
        for x in &self.balances {
            if x.locked.parse::<f64>().unwrap_or(0.0) == 0.0
                && x.free.parse::<f64>().unwrap_or(0.0) == 0.0
            {
                continue;
            }

            // This block is used to lock the balance to ensure fast lock
            // and release
            {
                let mut guard = balances.0.write().await;
                guard.insert(
                    x.asset.clone(),
                    Balance {
                        available: Decimal::from_str(&x.free).unwrap_or_default(),
                        locked: Decimal::from_str(&x.locked).unwrap_or_default(),
                    },
                );
            }
        }

        Ok(balances)
    }
}
