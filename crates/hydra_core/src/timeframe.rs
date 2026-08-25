use std::{fmt, str::FromStr};

use argus::errors::CustomErr;

#[derive(Debug, Clone)]
pub enum TimeFrame {
    OneMinute,
    FiveMinutes,
    ThirtyMinutes,
    OneHour,
    FourHour,
    OneDay,
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
    type Err = CustomErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "One Minute" => Ok(TimeFrame::OneMinute),
            "Five Minutes" => Ok(TimeFrame::FiveMinutes),
            "Thirty Minutes" => Ok(TimeFrame::ThirtyMinutes),
            "One Hour" => Ok(TimeFrame::OneHour),
            "Four Hour" => Ok(TimeFrame::FourHour),
            "One Day" => Ok(TimeFrame::OneDay),
            _ => Err(CustomErr::operation_ori(format!(
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
    pub fn get_period_minutes(&self) -> i32 {
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
    pub fn get_period_names1(&self) -> Box<str> {
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
