use argus::errors::ArgusErr;
use chrono::{DateTime, Days, Local};
use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512};
use std::{fmt, str::FromStr};

/// Various hasher types might be used in signing/encrypting
pub enum Hasher {
    Sha256,
    Sha512,
}

/// Various request types used to handle request to the server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    MAKER,
    TAKER,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    SELL,
    BUY,
}

#[derive(Debug, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<(Decimal, Decimal)>,
    pub asks: Vec<(Decimal, Decimal)>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Order {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    pub base: String,
    pub quote: String,
}

/// Store how long a time range will be applied. Useful to grab some data.
/// This will be very useful when we needs to grab some data.
/// We simply create the TimeRange that will be guaranteed in
/// chrono local DateTime.
#[derive(Debug, Default, Clone)]
pub struct TimeRange {
    pub from: DateTime<Local>,
    pub to: DateTime<Local>,
}

#[derive(Debug, Clone)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
    OneHour,
    FourHour,
    OneDay,
}

impl TimeRange {
    pub fn new(end_time: Option<DateTime<Local>>, lookback_days: u64) -> Result<Self, ArgusErr> {
        let to_time = match end_time {
            Some(time) => time,
            _ => chrono::Local::now(),
        };

        let mut a = Self::default();

        a.to = to_time;
        a.from = to_time
            .checked_sub_days(Days::new(lookback_days))
            .unwrap_or(to_time);

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

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TimeFrame::OneMinute => "One Minute",
            TimeFrame::FiveMinutes => "Five Minutes",
            TimeFrame::ThirtyMinutes => "Thirty Minutes",
            TimeFrame::OneHour => "One Hour",
            TimeFrame::FourHour => "Four hour",
            TimeFrame::OneDay => "One Day",
        };

        write!(f, "{}", s)
    }
}

impl FromStr for TimeFrame {
    type Err = ArgusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "One Minute" => Ok(TimeFrame::OneMinute),
            "Five Minutes" => Ok(TimeFrame::FiveMinutes),
            "Thirty Minutes" => Ok(TimeFrame::ThirtyMinutes),
            "One Hour" => Ok(TimeFrame::OneHour),
            "Four Hour" => Ok(TimeFrame::FourHour),
            "One Day" => Ok(TimeFrame::OneDay),
            _ => Err(ArgusErr::operation_ori(format!(
                "Failed to convert unknown string: {}",
                s
            ))),
        }
    }
}

impl TimeFrame {
    pub const ALL_VARIANTS: [TimeFrame; 6] = [
        TimeFrame::OneMinute,
        TimeFrame::FiveMinutes,
        TimeFrame::ThirtyMinutes,
        TimeFrame::OneHour,
        TimeFrame::FourHour,
        TimeFrame::OneDay,
    ];

    /// This is a function fo facilitate market that provides timeframe in minutes period in their API
    pub fn get_period_minute(&self) -> i32 {
        match self {
            TimeFrame::OneMinute => 1,
            TimeFrame::FiveMinutes => 5,
            TimeFrame::ThirtyMinutes => 30,
            TimeFrame::OneHour => 60,
            TimeFrame::FourHour => 240,
            TimeFrame::OneDay => 720,
        }
    }

    /// This is a function to facilitate market that provides their timframe in string
    pub fn get_period_name(&self) -> Box<str> {
        match self {
            TimeFrame::OneMinute => "1m".into(),
            TimeFrame::FiveMinutes => "5m".into(),
            TimeFrame::ThirtyMinutes => "30m".into(),
            TimeFrame::OneHour => "1h".into(),
            TimeFrame::FourHour => "4h".into(),
            TimeFrame::OneDay => "1d".into(),
        }
    }
}

/// Convert any struct into string data to be the request body.
pub async fn generate_request_body<P: serde::Serialize + Send + Sync + 'static>(
    payload: P,
) -> Result<Box<str>, ArgusErr> {
    match serde_qs::to_string(&payload) {
        Ok(p) => return Ok(Box::from(p)),
        Err(e) => return Err(ArgusErr::operation("Failed to build request body!", e)),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    pub struct DummyStruct {
        name: String,
        age: i64,
    }

    #[tokio::test]
    async fn generate_request_body_test() {
        let body = DummyStruct {
            name: "test".to_string(),
            age: 64,
        };

        let res = generate_request_body(body).await;

        assert!(res.is_ok());

        let parsed = res.unwrap();
        assert_eq!(parsed, Box::from("name=test&age=64"))
    }
}
