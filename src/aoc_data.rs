use crate::STAR_EMOJI;
use derive_more::Display;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd, Reverse};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use reqwest::header::COOKIE;

pub fn get_latest_local() -> AocData {
    let file = "latest.json";
    get_local_data(file)
}

pub fn get_local_data(file: &str) -> AocData {
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
    #[serde(rename(serialize = "members", deserialize = "members"))]
    players: HashMap<PlayerId, Player>,
}

impl AocData {
    pub fn scores_fmt(&self) -> String {
        let mut fmt_score = String::new();
        for (pos, (pl, score)) in self.scores().iter().enumerate() {
            fmt_score.push_str(&format!(
                "{0: <3} {1: <20} {2:<1}: {3:<5} ls: {4:<4}\n",
                pos + 1,
                pl,
                STAR_EMOJI,
                score.stars,
                score.local
            ));
        }
        fmt_score.push_str("* Local score (ls) för dag 1. ej inräknad. Klintan ska fixa.");
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

    fn player_ids(&self) -> HashSet<PlayerId> {
        self.players.keys().cloned().collect()
    }
    pub fn diff(&self, prev: &AocData) -> Option<Diff> {
        if self.latest_star() == prev.latest_star() && self.num_players() == prev.num_players() {
            None
        } else {
            let new_players: HashSet<PlayerId> = self.player_ids().difference(&prev.player_ids()).cloned().collect();
            let upd_players = self.players.keys().filter(|id| !new_players.contains(id))
                .filter(|id| {
                    let new_ts = self.players.get(id).unwrap().last_star_ts;
                    let prev_ts = prev.players.get(id).unwrap().last_star_ts;
                    new_ts != prev_ts})
                .map(|id| (self.players.get(id).unwrap(), prev.players.get(id).unwrap()));
            let new_stars = upd_players.map(|(new, prev)| (new.name.clone(), new.diff_stars(prev))).collect();
            let new_players = new_players.iter().map(|id| self.players.get(id).unwrap().name.clone()).collect();
            Some(Diff {new_players, new_stars})
        }
    }

    pub fn latest_star(&self) -> Option<TimeStamp> {
        self.players
            .iter()
            .filter_map(|(_, pl)| pl.last_star_ts)
            .max()
    }

    fn num_players(&self) -> usize {
        self.players.len()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Diff {
    new_players: Vec<String>,
    new_stars: HashMap<String, HashMap<u32, u32>>
}

impl Diff {
    pub fn fmt(self) -> String {
        let mut fmt_diff = String::new();
        for (pl, stars) in self.new_stars.iter() {
            for (day, sc) in stars {
            fmt_diff.push_str(&format!(
                "{0: <20} - Dag {1: <2}: {2:<2}\n",
                pl,
                day,
                if *sc == 1 {STAR_EMOJI.to_string()} else {format!("{}{}",STAR_EMOJI, STAR_EMOJI)},
            ));
        }
        }
        fmt_diff
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
        self.stars
            .cmp(&other.stars)
            .then(self.local.cmp(&other.local))
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct PlayerId(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    name: String,
    local_score: LocalScore,
    //TODO rename to latest star
    #[serde(deserialize_with = "de_timestamp")]
    last_star_ts: Option<TimeStamp>,
    stars: StarCount,
    completion_day_level: BTreeMap<u32, DayCompletion>,
}

impl Player {
    fn diff_stars(&self, prev: &Player) -> HashMap<u32, u32> {
        self.completion_day_level.iter().fold(HashMap::new(), |mut acc, (id, dc)| {
            if !prev.completion_day_level.contains_key(id) {
                let star_count = if dc.star_2.is_some() {2} else {1};
                acc.insert(*id, star_count);
            } else {
                let dc_prev = prev.completion_day_level.get(id).unwrap();
                if dc_prev.star_2.is_none() && dc.star_2.is_some() {
                    acc.insert(*id, 2);
                }
            }
            acc
        })
    }

    fn score(&self) -> Score {
        Score {
            stars: self.stars,
            local: self.local_score,
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
    #[serde(deserialize_with = "de_timestamp")]
    ts: Option<TimeStamp>,
}

#[derive(Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct StarCount(u32);

#[derive(Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
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
fn de_timestamp<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<TimeStamp>, D::Error> {
    match Value::deserialize(deserializer)? {
        Value::String(s) => {
            let ts = s.trim().parse::<i64>().map_err(|err| de::Error::custom(&format!("string parse: {}", err.to_string())))?;
            Ok(Some(TimeStamp(ts)))
        },
        Value::Number(ts) => match ts.as_i64() {
            Some(ts) => Ok(Some(TimeStamp(ts))),
            None => Err(de::Error::custom("i64 ts parsing")),
        }
        Value::Null => Ok(None),
        _ => Err(de::Error::custom("wrong type")),
    }
}

fn get_session_cookie() -> String {
    let token = env::var("AOC_SESSION").expect("Expected a token in the environment");
    format!("session={}", token)
}
