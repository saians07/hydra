/*
 * Handle all functionalities related to account
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::core::dtos::OrderPosition;

#[async_trait]
pub trait Account {
    async fn private_fetch_account_information<T: DeserializeOwned>(&self) -> Result<T, ArgusErr>;
    async fn private_fetch_open_orders(&self) -> Result<Vec<OrderPosition>, ArgusErr>;
}
