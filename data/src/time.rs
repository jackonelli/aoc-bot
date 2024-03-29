use crate::AocError;
use chrono::prelude::*;
use chrono::Local;
use derive_more::Display;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::time::{Duration, UNIX_EPOCH};

#[derive(
    Copy, Clone, Debug, Display, Hash, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize,
)]
pub struct Day(pub(crate) u32);

impl Day {
    pub fn try_new(day: u32) -> Result<Self, AocError> {
        if (1..=25_u32).contains(&day) {
            Ok(Day(day))
        } else {
            Err(AocError::Param {
                param: "Day".to_string(),
                val: day.to_string(),
                reason: "Day not between 1 and 25".to_string(),
            })
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct TimeStamp(pub(crate) u64);

impl TimeStamp {
    pub fn new(ts: u64) -> Self {
        TimeStamp(ts)
    }

    pub fn hour_and_minute(self) -> (u32, u32) {
        let dt: DateTime<Local> = self.into();
        (dt.hour(), dt.minute())
    }
}

impl TimeStamp {
    pub fn hour_and_minute_and_secs(self) -> (u32, u32, u32) {
        let dt: DateTime<Local> = self.into();
        (dt.hour(), dt.minute(), dt.second())
    }
}

impl std::fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (hour, min) = self.hour_and_minute();
        write!(f, "{}:{}", hour, min)
    }
}

impl From<TimeStamp> for DateTime<Local> {
    fn from(ts: TimeStamp) -> Self {
        let d = UNIX_EPOCH + Duration::from_secs(ts.0);
        DateTime::<Local>::from(d)
    }
}

pub(crate) fn sort_optional_ts(a: &Option<TimeStamp>, b: &Option<TimeStamp>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Special parsing of `TimeStamp`
///
/// The timestamps in the `completion_day_level` fields, a missing `get_star_ts` field are set to `null`,
/// whereas in the `last_star_ts` they are set to 0
/// Easiest to accept two match arms.
pub(crate) fn de_timestamp<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<TimeStamp, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(ts) => {
            let ts = ts
                .trim()
                .parse::<u64>()
                .map_err(|err| de::Error::custom(&format!("string parse: {}", err.to_string())))?;
            Ok(TimeStamp(ts))
        }
        Value::Number(ts) => match ts.as_i64() {
            Some(ts) => Ok(TimeStamp(ts as u64)),
            None => Err(de::Error::custom("u64 ts parsing")),
        },
        _ => Err(de::Error::custom("wrong type")),
    }
}

/// Special parsing of Option<[`TimeStamp`]>
///
/// The timestamps in the `completion_day_level` fields, a missing `get_star_ts` field are set to `null`,
/// whereas in the `last_star_ts` they are set to 0
/// Easiest to accept two match arms.
pub(crate) fn de_opt_timestamp<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<TimeStamp>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(ts) => {
            let ts = ts
                .trim()
                .parse::<u64>()
                .map_err(|err| de::Error::custom(&format!("string parse: {}", err.to_string())))?;
            Ok(Some(TimeStamp(ts)))
        }
        Value::Number(ts) => match ts.as_i64() {
            Some(0) => Ok(None),
            Some(ts) => Ok(Some(TimeStamp(ts as u64))),
            None => Err(de::Error::custom("u64 ts parsing")),
        },
        Value::Null => Ok(None),
        _ => Err(de::Error::custom("wrong type")),
    }
}
