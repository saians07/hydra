use std::{
    fmt::{self, Display},
    str::FromStr,
};

use argus::errors::ArgusErr;
use hmac::{Hmac, KeyInit, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512};

type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hasher {
    Sha256,
    Sha512,
}

#[derive(Debug, Clone, Default)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
    #[default]
    OneHour,
    FourHour,
    OneDay,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum TimeInForce {
    #[default]
    GTC, // good till cancelled
    IOC,
    FOK, // FIll or Kill
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub enum OrderType {
    MAKER,
    #[default]
    TAKER,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    SELL,
    #[default]
    BUY,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderStatus {
    #[default]
    Open,
    Closed,
    Canceled,
    Filled,
    PartiallyFilled,
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

impl Display for TimeFrame {
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

impl Display for OrderSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderSide::BUY => write!(f, "BUY"),
            OrderSide::SELL => write!(f, "SELL"),
        }
    }
}

impl Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderType::MAKER => write!(f, "LIMIT"),
            OrderType::TAKER => write!(f, "MARKET"),
        }
    }
}
