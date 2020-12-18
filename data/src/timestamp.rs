use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
#[derive(Copy, Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct TimeStamp(i64);

/// Special parsing of Option<[`TimeStamp`]>
///
/// The timestamps in the `completion_day_level` fields, a missing `get_star_ts` field are set to `null`,
/// whereas in the `last_star_ts` they are set to 0
/// Easiest to accept two match arms.
pub(crate) fn de_timestamp<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<TimeStamp>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(ts) => {
            let ts = ts
                .trim()
                .parse::<i64>()
                .map_err(|err| de::Error::custom(&format!("string parse: {}", err.to_string())))?;
            Ok(Some(TimeStamp(ts)))
        }
        Value::Number(ts) => match ts.as_i64() {
            Some(0) => Ok(None),
            Some(ts) => Ok(Some(TimeStamp(ts))),
            None => Err(de::Error::custom("i64 ts parsing")),
        },
        Value::Null => Ok(None),
        _ => Err(de::Error::custom("wrong type")),
    }
}
