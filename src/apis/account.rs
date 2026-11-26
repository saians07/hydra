/*
 * Handle all functionalities related to account
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::core::dtos::{OrderPosition, Pair};

#[async_trait]
pub trait Account {
    async fn private_fetch_account_information<T: DeserializeOwned>(&self) -> Result<T, ArgusErr>;
    async fn private_fetch_open_orders(
        &self,
        pair: Option<Pair>,
        recv_window: Option<i64>,
    ) -> Result<Vec<OrderPosition>, ArgusErr>;
    async fn private_fetch_order(
        &self,
        symbol: String,
        order_id: String,
    ) -> Result<OrderPosition, ArgusErr>;
}
