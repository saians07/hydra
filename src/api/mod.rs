// Handle unified API (trait) for every exchange
//
// Each exchange capability declared and stated explicitly here
// Every capability declared must be adopted. If somehow, an
// echange not having the capability, then `not_implemented!` macro
// can be used.

pub mod account;
pub mod data;
pub mod exhange;
pub mod request;
pub mod spot;
pub mod wallet;
