use argus::errors::ArgusErr;
use async_trait::async_trait;
use chrono::{DateTime, Local};

#[async_trait]
pub trait Exchange {
    async fn public_get_server_time(&self) -> Result<DateTime<Local>, ArgusErr>;
}
