use argus::errors::ArgusErr;
use async_trait::async_trait;

#[async_trait]
pub trait DataStream {
    async fn public_stream_spot_order_book(&self) -> Result<Box<str>, ArgusErr>;
    async fn public_stream_recent_trades(&self) -> Result<(), ()>;
    async fn private_stream_orders(&self) -> Result<(), ()>;
}
