/*
 * This traits provide a capability that must be had
 * by a balance manager to manage our balance.
 */
pub mod core;
pub mod exchanges;

#[cfg(feature = "indodax")]
pub use exchanges::indodax;
