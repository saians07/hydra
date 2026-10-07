use std::str::FromStr;

use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use polars::{df, frame::DataFrame};
use reqwest::{Client, header::HeaderMap};
use rust_decimal::Decimal;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};

use crate::{
    api::{
        account::Account,
        data::DataFetch,
        exhange::Exchange,
        request::Request,
        spot::Spot,
        wallet::{Balance, Balances, Wallet},
    },
    core::utils::{
        Hasher, Order, OrderBook, OrderSide, OrderType, Pair, RequestType, TimeFrame, TimeRange,
    },
};

pub struct Binance {
    pub id: Box<str>,
    pub name: Box<str>,
    api_key: SecretString,
    secret_key: SecretString,
    pub api_url_testing: Option<Box<str>>,
    pub private_api_url: Box<str>,
    pub public_api_url: Box<str>,
    pub private_ws_url: Box<str>,
    pub public_ws_url: Box<str>,
    client: Client,
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
        query: Option<&str>,
        headers: HeaderMap,
        need_signed: bool,
    ) -> Result<T, ArgusErr> {
        let mut full_url = format!("{}?{}", endpoint, query.unwrap_or(""));

        // println!("Full URL: {}", &full_url);

        match request_type {
            RequestType::GET => {
                if need_signed {
                    let signed_query = self.sign_request(query.unwrap()).await?;
                    full_url = full_url + format!("&signature={}", signed_query).as_str();
                }
                let result = self
                    .client
                    .get(full_url)
                    .header("Accept", "application/json")
                    .headers(headers);

                let req = result
                    .send()
                    .await
                    .unwrap()
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code", e))?;
                let result = req.json().await.map_err(|e| {
                    ArgusErr::operation("failed to convert Binance's response to json", e)
                })?;

                Ok(result)
            }
            RequestType::POST => {
                if need_signed {
                    let signed_body = self.sign_request(body.unwrap()).await?;
                    full_url = full_url + format!("signature={}", signed_body).as_str();
                }

                println!("Full URL: {}", &full_url);
                let result = self
                    .client
                    .post(full_url)
                    .header("Content-Type", "application/json")
                    .header("Accept", "application/json")
                    .headers(headers)
                    .body(body.unwrap_or("").to_string());

                let res = result
                    .send()
                    .await
                    .map_err(|e| ArgusErr::operation("Failed to request data from Binance", e))?
                    .error_for_status()
                    .map_err(|e| ArgusErr::operation("Error with status code.", e))?
                    .json()
                    .await
                    .map_err(|e| {
                        ArgusErr::operation("Failed to convert Binance's response to json", e)
                    })?;

                Ok(res)
            }
            _ => return Err(ArgusErr::operation_ori("Failed tp ")),
        }
    }
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

        let data = self
            .send_request::<T>(
                format!("{}{}", self.private_api_url, "/api/v3/account").as_str(),
                RequestType::GET,
                None,
                Some(&query),
                headers,
                true,
            )
            .await?;

        Ok(data)
    }
}

#[async_trait]
impl DataFetch for Binance {
    async fn public_fetch_order_book(&self, pair: Pair) -> Result<OrderBook, ArgusErr> {
        let endpoint = format!("{}{}", self.public_api_url, "/api/v3/depth");
        let headers = HeaderMap::new();
        let symbol = format!("symbol={}{}", pair.base, pair.quote);

        let res = self
            .send_request::<OrderBook>(
                &endpoint,
                RequestType::GET,
                None,
                Some(&symbol),
                headers,
                false,
            )
            .await?;

        Ok(res)
    }

    async fn public_fetch_ohlcv(
        &self,
        time_frame: TimeFrame,
        time_range: TimeRange,
        pair: Pair,
    ) -> Result<DataFrame, ArgusErr> {
        let endpoint = format!("{}{}", self.public_api_url, "/api/v3/klines");
        let headers = HeaderMap::new();
        let query = format!(
            "symbol={}{}&interval={}&startTime={}&endTime={}",
            pair.base,
            pair.quote,
            time_frame.get_period_name(),
            time_range.from.with_timezone(&Utc).timestamp() * 1000,
            time_range.to.with_timezone(&Utc).timestamp() * 1000,
        );
        let res = self
            .send_request::<Vec<BNBOhlcv>>(
                &endpoint,
                RequestType::GET,
                None,
                Some(&query),
                headers,
                false,
            )
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
    async fn get_server_time(&self) -> Result<DateTime<Local>, ArgusErr> {
        let endpoint = format!("{}{}", self.public_api_url, "/api/v3/time");
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse().unwrap());

        let res = self
            .send_request::<serde_json::Value>(
                &endpoint,
                RequestType::GET,
                None,
                None,
                headers,
                false,
            )
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
impl Spot for Binance {
    async fn private_send_order(
        &self,
        order_type: OrderType,
        order_side: OrderSide,
        order: Order,
    ) -> Result<bool, ArgusErr> {
        let endpoint = format!("{}{}", self.private_api_url, "/api/v3/order");
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );
        let payload = format!("symbol={}", order.pair);

        Err(ArgusErr::BADREQUEST)
    }
}

#[async_trait]
impl Wallet for Binance {
    async fn private_fetch_balances(&self) -> Result<Balances, ArgusErr> {
        let account_info = self
            .private_fetch_account_information::<BNBAccountInfo>()
            .await?;

        let balance = account_info.get_balance().await;

        balance
    }
}

impl Binance {
    pub fn new(api_key: SecretString, secret_key: SecretString) -> Self {
        Self {
            id: "gate".into(),
            name: "Gate".into(),
            api_key,
            secret_key,
            api_url_testing: None,
            private_api_url: "https://api.binance.com".into(),
            public_api_url: "https://api.binance.com".into(),
            private_ws_url: "".into(),
            public_ws_url: "".into(),
            client: Client::new(),
        }
    }

    /// Specific BNB API to get assets that convertable to BNB and or BTC.
    /// This is usually for small amount of assets (dust)
    pub async fn dust_convertible_assets(&self) -> Result<(), ArgusErr> {
        let body = format!("timestamp={}", chrono::Local::now().timestamp_millis());
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );

        let data: Value = self
            .send_request(
                format!("{}{}", self.private_api_url, "/sapi/v1/asset/dust-btc").as_str(),
                RequestType::POST,
                Some(&body),
                None,
                headers,
                true,
            )
            .await?;

        println!("Result: {:#?}", data);

        Ok(())
    }
}

//###################################################//
// Starting from here, we will have private struct to handle various Binance response
//###################################################//
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

#[derive(Debug, Serialize, Deserialize)]
struct BNBAccountInfo {
    #[serde(rename = "makerCommission")]
    maker_commission: f64,
    #[serde(rename = "takerCommission")]
    taker_commission: f64,
    pub balances: Vec<BNBBalance>,
    #[serde(rename = "canTrade")]
    can_trade: bool,
    #[serde(rename = "accountType")]
    account_type: String,
    permissions: Vec<String>,
}

impl BNBAccountInfo {
    /// Exclusively used to extract balances from account information fetched from Binance server.
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
                        available: Decimal::from_str(&x.locked).unwrap_or_default(),
                        locked: Decimal::from_str(&x.free).unwrap_or_default(),
                    },
                );
            }
        }

        Ok(balances)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BNBBalance {
    asset: Box<str>,
    free: Box<str>,
    locked: Box<str>,
}
