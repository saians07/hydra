/*
 * Handle all kinds of spot transaction
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;

use crate::core::dtos::{OrderRequest, OrderResponse};

#[async_trait]
pub trait SpotOrder {
    async fn private_send_order_request(
        &self,
        order: OrderRequest,
    ) -> Result<OrderResponse, ArgusErr>;
}
