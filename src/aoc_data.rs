use std::env;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::prelude::*;
use reqwest::header::COOKIE;

pub async fn get_aoc_data() -> Result<AocData, reqwest::Error> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://adventofcode.com/2020/leaderboard/private/view/152507.json")
        .header(COOKIE, get_session_cookie())
        .send()
        .await?
        .text()
        .await?;
    Ok(serde_json::from_str(&res).unwrap())

}

#[derive(Debug, Deserialize, Serialize)]
pub struct AocData {
    event: String,
    #[serde(deserialize_with = "de_player_id")]
    owner_id: PlayerId,
    #[serde(rename(deserialize = "members"))]
    players: HashMap<PlayerId, Player>,
}

impl AocData {
    pub fn order(&self) -> Vec<(String, StarCount)> {
        let mut score: Vec<_> = self
            .players
            .iter()
            .map(|(_, pl)| (pl.name.clone(), pl.stars))
            .collect();
        score.sort_unstable_by_key(|(_, stars)| -(stars.0 as i32));
        score
    }

    fn _num_players(&self) -> usize {
        self.players.len()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct PlayerId(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    name: String,
    local_score: Score,
    global_score: Score,
    #[serde(deserialize_with = "de_timestamp")]
    last_star_ts: Option<TimeStamp>,
    stars: StarCount,
    completion_day_level: BTreeMap<u32, DayCompletion>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DayCompletion {
    #[serde(rename = "1")]
    star_1: StarProgress,
    #[serde(rename = "2")]
    star_2: Option<StarProgress>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct StarProgress {
    #[serde(rename = "get_star_ts")]
    ts: TimeStamp,
}

#[derive(
    Copy, Clone, Debug, Deserialize, Serialize, cmp::Ord, cmp::PartialOrd, cmp::Eq, cmp::PartialEq,
)]
pub struct StarCount(u32);

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct Score(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TimeStamp(String);

fn de_player_id<'de, D: Deserializer<'de>>(deserializer: D) -> Result<PlayerId, D::Error> {
    let raw = match Value::deserialize(deserializer)? {
        Value::String(s) => s.parse::<u32>().map_err(de::Error::custom)?,
        Value::Number(num) => {
            num.as_u64()
                .ok_or_else(|| de::Error::custom("Invalid number"))? as u32
        }
        _ => return Err(de::Error::custom("wrong type")),
    };
    Ok(PlayerId(raw))
}

/// If no star, the `last_star_ts` field is set to 0, not `null`. Requires special handling.
fn de_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TimeStamp>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(s) => Ok(Some(TimeStamp(s))),
        Value::Number(_) => Ok(None),
        _ => Err(de::Error::custom("wrong type")),
    }
}

fn get_session_cookie() -> String {
    let token = env::var("AOC_SESSION").expect("Expected a token in the environment");
    format!("session={}", token)
}
