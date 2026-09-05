use argus::errors::ArgusErr;
use chrono::{DateTime, Days, Local};
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

#[derive(Debug, Default)]
pub struct TimeRange {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

impl TimeRange {
    pub fn create_time_range(&self, lookback_days: u64) -> Result<Self, ArgusErr> {
        let mut a = Self::default();

        let current_time = Local::now();
        a.to = current_time;
        a.from = current_time
            .checked_sub_days(Days::new(lookback_days))
            .unwrap_or(current_time);

        Ok(a)
    }

    pub fn to_timestamp(&self) -> Result<(i64, i64), ArgusErr> {
        let to: i64 = self.to.timestamp();
        let from: i64 = self.from.timestamp();

        Ok((to, from))
    }

    pub fn to_str(&self) -> Result<(Box<str>, Box<str>), ArgusErr> {
        let (to, from) = self.to_timestamp()?;

        Ok((to.to_string().into(), from.to_string().into()))
    }
}

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

impl Hasher {
    pub fn hash(&self, raw: &str, key: &str) -> Result<Box<str>, ArgusErr> {
        match self {
            Hasher::Sha256 => {
                let mut mac = HmacSha256::new_from_slice(key.as_bytes())
                    .map_err(|e| ArgusErr::operation("Failed to sign the data.", e))?;
                mac.update(raw.as_bytes());
                let result = mac.finalize().into_bytes();

                Ok(Box::from(hex::encode(result)))
            }
            Hasher::Sha512 => {
                let mut mac = HmacSha512::new_from_slice(key.as_bytes())
                    .map_err(|e| ArgusErr::operation("Failed to sign the data.", e))?;
                mac.update(raw.as_bytes());
                let result = mac.finalize().into_bytes();

                Ok(Box::from(hex::encode(result)))
            }
        }
    }
}
