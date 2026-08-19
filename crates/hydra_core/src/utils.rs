use argus::errors::CustomErr;
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Sha256, Sha512};

pub enum Hasher {
    Sha256,
    Sha512,
}

pub enum RequestType {
    GET,
    POST,
}

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

pub fn hmac_256(raw: &str, key: &str) -> Result<Box<str>, CustomErr> {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|e| CustomErr::operation("Failed to sign the data.", e))?;
    mac.update(raw.as_bytes());
    let result = mac.finalize().into_bytes();

    Ok(Box::from(hex::encode(result)))
}

pub fn hmac_512(raw: &str, key: &str) -> Result<Box<str>, CustomErr> {
    let mut mac = HmacSha512::new_from_slice(key.as_bytes())
        .map_err(|e| CustomErr::operation("Failed to sign the data.", e))?;
    mac.update(raw.as_bytes());
    let result = mac.finalize().into_bytes();

    Ok(Box::from(hex::encode(result)))
}
