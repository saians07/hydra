/*
 * Two data fetching mechanism are employed here:
 *      - Streaming: Protocol-based real-time data (e.g. WebSocket).
 *      - Fetching: Request-based API calls (e.g. REST API)
 *
 * You can fetch the following data:
 *      - Order Book
 *      - Market Trades
 *      - Your account trades
 *      - Order status
 *      - Kline data
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use polars::frame::DataFrame;

use crate::core::utils::{OrderBook, Pair, TimeFrame, TimeRange};

#[async_trait]
pub trait DataStream {
    async fn public_stream_spot_order_book(&self) -> Result<Box<str>, ArgusErr>;
    async fn public_stream_spot_recent_trades(&self) -> Result<(), ()>;
    async fn private_stream_orders(&self) -> Result<(), ()>;
}

#[async_trait]
pub trait DataFetch {
    async fn public_fetch_order_book(&self, pair: Pair) -> Result<OrderBook, ArgusErr>;
    async fn public_fetch_ohlcv(
        &self,
        time_frame: TimeFrame,
        time_range: TimeRange,
        pair: Pair,
    ) -> Result<DataFrame, ArgusErr>;
    // async fn public_fetch_recent_market_trades(&self) -> Result<(), ()>;
    // async fn private_fetch_trades(&self) -> Result<(), ()>;
}
