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
        trades::spot::SpotOrder,
        wallet::{Balance, Balances, Wallet},
    },
    core::{
        dtos::{OrderBook, OrderPosition, OrderRequest, OrderResponse, Pair, TimeRange},
        enums::{Hasher, RequestType, TimeFrame},
        helper::generate_request_body,
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
        let query = generate_timestamp_query().await;
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

    async fn private_fetch_open_orders(
        &self,
        pair: Option<Pair>,
        recv_window: Option<i64>,
    ) -> Result<Vec<OrderPosition>, ArgusErr> {
        let mut query = generate_timestamp_query().await;
        if let Some(pr) = pair {
            query += format!("&symbol={}", pr.create_type_1()).as_str()
        }
        if let Some(recv) = recv_window {
            query += format!("&recvWindow={}", recv).as_str()
        }
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );

        let signed_query = self.sign_request(&query).await?;

        let endpoint = format!(
            "{}{}{}&signature={}",
            self.private_api_url, "/api/v3/openOrders?", query, signed_query
        );

        self.send_request::<Vec<OrderPosition>>(&endpoint, RequestType::GET, None, headers)
            .await
    }

    async fn private_fetch_order(
        &self,
        symbol: String,
        order_id: String,
    ) -> Result<OrderPosition, ArgusErr> {
        unimplemented!()
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
    async fn public_fetch_server_time(&self) -> Result<DateTime<Local>, ArgusErr> {
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

                let response = request.send().await.map_err(|e| {
                    ArgusErr::operation("Failed to post request to Binance server!", e)
                })?;

                if !response.status().is_success() {
                    let status = response.status();
                    let error_body = response.text().await.unwrap_or_default();
                    return Err(ArgusErr::operation_ori(format!(
                        "Binance returned {}: {}",
                        status, error_body
                    )));
                }

                response.json().await.map_err(|e| {
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
impl SpotOrder for Binance {
    async fn private_send_order(&self, order: OrderRequest) -> Result<OrderResponse, ArgusErr> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        headers.insert(
            "X-MBX-APIKEY",
            self.api_key.expose_secret().parse().unwrap(),
        );
        let symbol = format!(
            "{}{}",
            order.pair.base.to_owned(),
            order.pair.quote.to_owned()
        );
        let order_data = BNBOrderRequest {
            symbol: symbol.into(),
            side: order.order_side.to_string(),
            order_type: order.order_type.to_string(),
            timestamp: chrono::Local::now().timestamp_millis(),
            quantity: order.base_amount,
            quote_order_quantity: order.quote_amount,
            price: order.expected_price,
            client_order_id: order.client_order_id.to_owned(),
            ..Default::default()
        };
        let body = generate_request_body(order_data).await?;
        let signed_body = self.sign_request(body.as_ref()).await?;
        let full_body = format!("{}&signature={}", body, signed_body);

        let endpoint = format!("{}{}", self.private_api_url, "/api/v3/order");

        let bnb_resp = self
            .send_request::<BNBOrderResponse>(
                &endpoint,
                order.request_type,
                Some(&full_body),
                headers,
            )
            .await?;

        let resp = OrderResponse {
            pair: order.pair,
            order_id: bnb_resp.order_id,
            market_id: self.id.to_owned(),
            order_type: order.order_type,
            order_side: order.order_side,
            trade_id: bnb_resp.fills.iter().map(|x| x.trade_id).collect(),
            client_order_id: order.client_order_id,
        };
        Ok(resp)
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

async fn generate_timestamp_query() -> String {
    format!("timestamp={}", chrono::Local::now().timestamp_millis())
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

#[derive(Debug, Default, Serialize, Deserialize)]
struct BNBOrderRequest {
    symbol: Box<str>,
    side: String,
    #[serde(rename = "type")]
    order_type: String,
    timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<Decimal>,
    #[serde(rename = "quoteOrderQty", skip_serializing_if = "Option::is_none")]
    quote_order_quantity: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<Decimal>,
    #[serde(rename = "newClientOrderId", skip_serializing_if = "Option::is_none")]
    client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trailing_data: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recv_window: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BNBOrderResponse {
    #[serde(rename = "orderId")]
    order_id: i64,
    #[serde(rename = "clientOrderId")]
    client_order_id: String,
    #[serde(rename = "cummulativeQuoteQty")]
    quote_quantity: Decimal,
    #[serde(rename = "executedQty")]
    executed_quantity: Decimal,
    fills: Vec<BinanceFillOrder>,
    #[serde(rename = "origQty")]
    original_quantity: Decimal,
    #[serde(rename = "origQuoteOrderQty")]
    original_quote_quantity: Decimal,
    price: Decimal,
    status: BNBOrderStatus,
    #[serde(rename = "transactTime")]
    transaction_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BinanceFillOrder {
    commission: Decimal,
    #[serde(rename = "commissionAsset")]
    commission_asset: String,
    price: Decimal,
    qty: Decimal,
    #[serde(rename = "tradeId")]
    trade_id: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BNBOrderStatus {
    #[default]
    New,
    PendingNew,
    Filled,
    PartiallyFilled,
    Canceled,
    PendingCancel,
    Rejected,
    Expired,
    ExpiredInMatch,
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
