/*
 * Handle any kind of account operations
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait Account {
    /// Fetch account information from the server
    ///
    /// # Returns
    ///
    /// Any object that implement `serde::DeserializeOwned`
    async fn private_fetch_account_information<T: DeserializeOwned>(&self) -> Result<T, ArgusErr>;
}
