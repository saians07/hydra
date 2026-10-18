/*
 * All of the following steps are executed in Garuda
 * - Initialize market [Hydra]
 * - Get all allowed currency and transacted currency [Iris]
 * - Fetch orderbook data [Hydra]
 * - Check if we need to modify selling order, or we need to cut loss the position [Kirin]
 * - Fetch OHLCV data [Hydra]
 * - Check if there is a buy signal from the market [Kirin]
 * - If there is no buy signal, then skip
 * - If there is a buy signal, we need to check:
 *      * Our balance availability [Hydra]
 *      * Open position is less than the threshold [Iris]
 *      * Buying price difference greater than threshold for any open position [Kirin]
 *      * Detect price crash [Kirin]
 *      * There is lower price rejection found (current close higher than lower price) [Kirin]
 * - If passed, then execute the buy command
 * - Once executed, distribute the sell using scaled order to achieve x% profit [Kirin]
 * - Calculate current open position and adjust all trading configuration [Kirin]
 */

#[cfg(feature = "binance")]
use argus::errors::ArgusErr;
use hydra::{api::wallet::Balances, core::utils::OrderBook};
#[cfg(feature = "binance")]
use hydra::{
    api::{data::DataFetch, spot::Spot, wallet::Wallet},
    core::utils::{OrderRequest, OrderSide, OrderStatus, OrderType, Pair},
    exchanges::binance::Binance,
};

#[cfg(feature = "binance")]
#[tokio::main]
async fn main() -> Result<(), ArgusErr> {
    // use rust_decimal::{Decimal, prelude::*};
    let binance = Binance::new(
        "z7gkd0DqdGj5bAa8ahpFZjcShUO1KIZc0Zikvf6qdBtBY0VR1SWOQijtqlwbXA9T".into(),
        "bEfXGDdcQxNddz3Onxi7E3yBXP3AXhJ9CnFjOosXHl7h0SFk3bSkZpMyWXkwRHcr".into(),
    );
    let mut markets: Vec<Market> = Vec::new();
    markets.push(Market::Binance(binance));
    let mut pairs: Vec<Pair> = Vec::new();

    // binance.private_fetch_account_information().await?;
    let btc_usdt = Pair::new("BTC".into(), "USDT".into());
    pairs.push(btc_usdt);

    for market in &markets {
        for pair in &pairs {
            let a = market.private_fetch_balances().await?;
            println!("The balance is {:#?}", a);

            let ob = market.public_fetch_order_book(pair).await?;
            println!("The orderbook is {:#?}", ob);
        }
    }
    // let ohlcv = binance
    //     .public_fetch_ohlcv(
    //         hydra::core::utils::TimeFrame::OneHour,
    //         hydra::core::utils::TimeRange::new(Some(chrono::Local::now()), 4u64)?,
    //         pair.clone(),
    //     )
    //     .await?;
    // println!("The OHLCV is: {:#?}", ohlcv.head(Some(4)));
    // let ord = OrderRequest {
    //     pair: pair,
    //     market_id: binance.id.clone(),
    //     base_amount: Decimal::from_f64(0.00009),
    //     order_type: OrderType::TAKER,
    //     order_side: OrderSide::BUY,
    //     order_status: OrderStatus::Open,
    //     client_order_id: 12345.into(),
    //     expected_price: None,
    //     quote_amount: None,
    // };
    // let resp = binance.private_send_order(ord.clone()).await?;
    // println!("The order response: {:#?}", resp);
    Ok(())
}

enum Market {
    Binance(Binance),
}

impl Market {
    pub async fn private_fetch_balances(&self) -> Result<Balances, ArgusErr> {
        match self {
            Market::Binance(b) => b.private_fetch_balances().await,
        }
    }

    pub async fn public_fetch_order_book(&self, pair: &Pair) -> Result<OrderBook, ArgusErr> {
        match self {
            Market::Binance(b) => b.public_fetch_order_book(pair).await,
        }
    }
}
