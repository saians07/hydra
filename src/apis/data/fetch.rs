use argus::errors::ArgusErr;
use async_trait::async_trait;
use polars::frame::DataFrame;

use crate::core::{
    dtos::{OrderBook, Pair, TimeRange},
    enums::TimeFrame,
};

#[async_trait]
pub trait DataFetch {
    async fn public_fetch_order_book(&self, pair: &Pair) -> Result<OrderBook, ArgusErr>;
    async fn public_fetch_ohlcv(
        &self,
        time_frame: TimeFrame,
        time_range: TimeRange,
        pair: &Pair,
    ) -> Result<DataFrame, ArgusErr>;
}
