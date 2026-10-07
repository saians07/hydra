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
#[cfg(feature = "binance")]
use hydra::{
    api::{data::DataFetch, wallet::Wallet},
    core::utils::Pair,
    exchanges::binance::Binance,
};

#[cfg(feature = "binance")]
#[tokio::main]
async fn main() -> Result<(), ArgusErr> {
    let binance = Binance::new(
        "z7gkd0DqdGj5bAa8ahpFZjcShUO1KIZc0Zikvf6qdBtBY0VR1SWOQijtqlwbXA9T".into(),
        "bEfXGDdcQxNddz3Onxi7E3yBXP3AXhJ9CnFjOosXHl7h0SFk3bSkZpMyWXkwRHcr".into(),
    );
    // let pairs = vec![Pair {
    //     base: "BTC".to_string(),
    //     quote: "USDT".to_string(),
    // }];
    // binance.private_fetch_account_information().await?;
    let pair = Pair {
        base: "BTC".to_string(),
        quote: "USDT".to_string(),
    };
    let a = binance.private_fetch_balances().await?;
    let ob = binance.public_fetch_order_book(pair.clone()).await?;
    let ohlcv = binance
        .public_fetch_ohlcv(
            hydra::core::utils::TimeFrame::OneHour,
            hydra::core::utils::TimeRange::new(Some(chrono::Local::now()), 4u64)?,
            pair,
        )
        .await?;
    // println!("The USDT Balances are: {:#?}", a.0.read().await);
    // println!("The order book is: {:#?}", ob);
    println!("The OHLCV is: {:#?}", ohlcv.head(Some(4)));
    Ok(())
}
