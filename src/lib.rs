/*
 * This traits provide a capability that must be had
 * by a balance manager to manage our balance.
 */
pub mod api;
pub mod core;
pub mod websocket;

pub mod exchanges {
    #[cfg(feature = "binance")]
    pub mod binance;

    #[cfg(feature = "safe_trade")]
    pub mod safe_trade;
}
