use anyhow::Result;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serenity::model::id::{ApplicationId, ChannelId};
use std::fs::File;
use std::io::Read;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AocBotConfig {
    #[serde(deserialize_with = "de_tokio_duration")]
    /// Duration in seconds
    pub api_delay: Duration,
    pub token: String,
    pub application_id: ApplicationId,
    pub channel_id: ChannelId,
    pub aoc_cookie: String,
}

impl AocBotConfig {
    pub fn from_config(config: &str) -> Result<AocBotConfig> {
        let mut file = File::open(config)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(serde_json::from_str(&contents)?)
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Read;

    use super::*;

    #[test]
    fn parse_sample_config() {
        let mut file = File::open("../sample_config.json").expect("File open fail");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("File read fail");
        let config: AocBotConfig = serde_json::from_str(&contents).expect("Parse fail");
        assert_eq!(config.api_delay, Duration::from_secs(901));
        assert_eq!(config.token, String::from("secret-token"));
        assert_eq!(config.application_id, 1);
        assert_eq!(config.channel_id, 2);
    }
}

/// Special parsing of tokio::Duration
///
/// The same integer can create many durations (from_secs, from_millis, et c.).
/// Need custom deserialiser to select one (here: seconds.).
fn de_tokio_duration<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::Number(ts) => match ts.as_u64() {
            Some(ts) => Ok(Duration::from_secs(ts)),
            None => Err(de::Error::custom("u64 ts parsing")),
        },
        _ => Err(de::Error::custom("wrong type")),
    }
}
