use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{DateTime, Local};

#[async_trait]
pub trait Exchange {
    // async fn exchnage_info<T: DeserializeOwned>(&self) -> Result<T, ArgusErr>;
    /// Get the server timestamp for the current time.
    async fn get_server_time(&self) -> Result<DateTime<Local>, ArgusErr>;
}
