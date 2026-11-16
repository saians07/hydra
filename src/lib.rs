pub mod apis;
pub mod core;
pub mod exchanges {
    #[cfg(feature = "binance")]
    pub mod binance;
}
