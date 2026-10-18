/*
 * Handle all kinds of spot transactions sent through API
 */

use argus::errors::ArgusErr;
use async_trait::async_trait;

use crate::core::utils::{OrderRequest, OrderResponse};

#[async_trait]
pub trait Spot {
    /// Send order into server
    ///
    /// Spot order basically can be a market order or a limit order.
    /// Market order will be executed immediately using the current price
    /// while limit order will be executed on the ordered price.
    /// Spot order has two types of side, Sell of Buy. Either you want to sell
    /// or you want to buy.
    /// Since, each market might have different order parameters and it is difficult to
    /// use variadic in rust, we catch all possible paramters under Order struct.
    async fn private_send_order(&self, order: OrderRequest) -> Result<OrderResponse, ArgusErr>;
}
