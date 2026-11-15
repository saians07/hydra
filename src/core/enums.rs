use argus::errors::ArgusErr;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Clone, PartialEq)]
pub enum Hasher {
    Sha256,
    Sha512,
}

impl Hasher {
    pub fn hash(&self, raw_string: &str, key: &str) -> Result<Box<str>, ArgusErr> {
        match self {
            Hasher::Sha256 => {
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                    .map_err(|e| ArgusErr::operation("Failed to hash the data.", e))?;
                mac.update(raw_string.as_bytes());
                let result = mac.finalize().into_bytes();

                Ok(Box::from(hex::encode(result)))
            }
            Hasher::Sha512 => {
                let mut mac = HmacSha512::new_from_slice(key.as_bytes())
                    .map_err(|e| ArgusErr::operation("Failed to hash the data.", e))?;
                mac.update(raw_string.as_bytes());
                let result = mac.finalize().into_bytes();

                Ok(Box::from(hex::encode(result)))
            }
        }
    }
}
