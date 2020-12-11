use std::env;
use derive_more::Display;
use std::fs::File;
use std::io::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp::{Reverse, Ordering, PartialOrd, Ord, PartialEq, Eq};
use std::collections::{BTreeMap, HashMap};
//use std::fs::File;
//use std::io::prelude::*;
use reqwest::header::COOKIE;

pub fn get_latest_local() -> AocData {
    let file = "latest.json";
    let mut file = File::open(file).expect("Opening file error");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Read to string error");
    serde_json::from_str(&contents).expect("parse_error")
}

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
    #[serde(rename(serialize="members", deserialize = "members"))]
    players: HashMap<PlayerId, Player>,
}

impl AocData {

    pub fn scores_fmt(&self) -> String {
    let mut fmt_score = String::new();
    for (pos, (pl, score)) in self.scores().iter().enumerate() {
        fmt_score.push_str(&format!("{}. {}: Stars: {}, Local: {}\n", pos+1, pl, score.stars, score.local));
    }
    fmt_score
    }

    pub fn scores(&self) -> Vec<(String, Score)> {
        let mut scores: Vec<(String, Score)> = self
            .players
            .iter()
            .map(|(_, pl)| (pl.name.clone(), pl.score()))
            .collect();
        scores.sort_unstable_by_key(|(_pl, score)| Reverse(*score));
        scores
    }

    pub fn latest_star(&self) -> Option<TimeStamp> {
        self.players.iter().filter_map(|(_, pl)| pl.last_star_ts.clone().or(None)).max()
        //let tss = self.players.iter().map(|(_, pl)| (pl.name.clone(), pl.last_star_ts));
        //for ts in tss.clone() {
        //println!("{:?}", ts);
        //}
        //None

    }

    fn _num_players(&self) -> usize {
        self.players.len()
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Score {
    pub stars: StarCount,
    pub local: LocalScore,
}


impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stars.cmp(&other.stars).then(self.local.cmp(&other.local))
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct PlayerId(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    name: String,
    local_score: LocalScore,
    //#[serde(deserialize_with = "de_timestamp")]
    last_star_ts: Option<TimeStamp>,
    stars: StarCount,
    completion_day_level: BTreeMap<u32, DayCompletion>,
}

impl Player {
    fn score(&self) -> Score {
        Score {
            stars: self.stars,
            local: self.local_score
        }
    }
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
    //#[serde(deserialize_with = "de_timestamp")]
    ts: Option<TimeStamp>,
}

#[derive(
    Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct StarCount(u32);

#[derive(
    Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct LocalScore(u32);

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Ord, PartialOrd, PartialEq, Eq)]
pub struct TimeStamp(i64);

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

///// If no star, the `last_star_ts` field is set to 0, not `null`. Requires special handling.
///// It seems to have been fixed now, but I'll keep both match arms to be safe.
//fn de_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TimeStamp>, D::Error> {
//    match Value::deserialize(deserializer)? {
//        Value::String(s) => {
//            let ts = s.trim().parse::<i64>().map_err(|err| de::Error::custom(&format!("string parse: {}", err.to_string())))?;
//            Ok(Some(TimeStamp(ts)))
//        },
//        Value::Number(_) => Ok(None),
//        Value::Null => Ok(None),
//        _ => Err(de::Error::custom("wrong type")),
//    }
//}

fn get_session_cookie() -> String {
    let token = env::var("AOC_SESSION").expect("Expected a token in the environment");
    format!("session={}", token)
}
