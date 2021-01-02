//! # Advent of Code data
//!
//! Provides a strictly typed data schema and logic for the [Advent of Code](https://adventofcode.com/) competition API.
mod time;
pub mod score;
pub mod diff;
use crate::time::{Day, TimeStamp, de_timestamp, de_opt_timestamp};
use crate::score::{Score, StarCount, LocalScore, GlobalScore};
use crate::diff::{Diff, NewStars};
use reqwest::header::COOKIE;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp::{Eq, PartialEq, Reverse};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use thiserror::Error;

/// For nice formatting
pub const STAR_SYMBOL: char = '\u{2B50}';

/// Representation of private leaderboard data return from the AoC API
#[derive(Debug, Deserialize, Serialize)]
pub struct AocData {
    /// Name of the event, e.g. '2020'
    event: String,
    /// Id. of the player hosting the private leaderboard
    #[serde(deserialize_with = "de_player_id")]
    owner_id: PlayerId,
    #[serde(rename = "members")]
    /// Collection of players in the private leaderboard and their progress.
    players: HashMap<PlayerId, Player>,
}

/// Player data
///
/// Holds the chosen display name and the current progress for all days.
/// Remaining fields are pre-computed metrics, all of which can be inferred from the
/// `completion_day_level` field.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct Player {
    /// Display name
    name: String,
    /// Progress for each day
    completion_day_level: BTreeMap<Day, DayCompletion>,
    local_score: LocalScore,
    global_score: GlobalScore,
    #[serde(deserialize_with = "de_opt_timestamp")]
    last_star_ts: Option<TimeStamp>,
    stars: StarCount,
}

impl AocData {
    pub fn scores_fmt(&self) -> String {
        let mut fmt_score = String::new();
        for (pos, (pl, score)) in self.scores().iter().enumerate() {
            fmt_score.push_str(&format!(
                "{0: <3} {1: <20} {2:<1}: {3:<5} ls: {4:<4}\n",
                pos + 1,
                pl,
                STAR_SYMBOL,
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

    fn player_ids(&self) -> HashSet<&PlayerId> {
        self.players.keys().collect()
    }

    /// Aggregate possible diff
    ///
    /// Compare `self` to a previous data point `prev`.
    /// If there are more recent stars or a change of players, return `Some([Diff])`.
    /// Otherwise return `None`.
    ///
    /// TODO: Correct check for change in players. Add field `lost_players` in [`Diff`]
    ///
    /// Never panics: unwrapping the access of `self.players[id]` is fine since `id` is in the
    /// set of `new_players` which is a subset of `self.players`.
    pub fn diff(&self, prev: &AocData) -> Option<Diff> {
        // Incorrect check since the leaderboard could change such that the players are
        // different but have the same total number of players.
        if self.latest_star() == prev.latest_star() && self.num_players() == prev.num_players() {
            None
        } else {
            let new_players = self.player_ids().difference(&prev.player_ids()).cloned().collect();
            let upd_players = self.updated_players(prev, &new_players);
            let new_stars = upd_players
                .map(|(new, prev)| (new.name.clone(), new.diff_stars(prev)))
                .collect();
            let new_players = new_players
                .into_iter()
                .map(|id| self.players.get(&id).unwrap().clone())
                .collect();
            Some(Diff {
                new_players,
                new_stars,
            })
        }
    }

    /// Updated players
    ///
    /// Get an iterator over players that both:
    /// a) are not new, compared to `prev.players`. I.e. players that are in both `prev.players` and `self.players`
    /// b) have different timestamps for the last star.
    ///
    /// Never panics: unwrapping the access of `prev.players[id]` is fine since `id` is in the
    /// set of ids which comes from: `self.players setminus new_players`
    fn updated_players<'a>(
        &'a self,
        prev: &'a AocData,
        new_players: &'a HashSet<&'a PlayerId>,
    ) -> impl Iterator<Item = (&'a Player, &'a Player)> {
        self.players
            .iter()
            .filter(move |(id, _player)| !new_players.contains(id))
            .filter(move |(id, player)| {
                let new_ts = player.last_star_ts();
                let prev_ts = prev.players.get(id).unwrap().last_star_ts();
                new_ts != prev_ts
            })
            .map(move |(id, player)| (player, prev.players.get(id).unwrap()))
    }

    pub fn latest_star(&self) -> Option<TimeStamp> {
        self.players
            .iter()
            .filter_map(|(_, pl)| pl.last_star_ts())
            .max()
    }

    fn num_players(&self) -> usize {
        self.players.len()
    }

    pub fn write_to_file(&self, file: &str) -> Result<(), AocError> {
        serde_json::to_writer(&File::create(file)?, self).map_err(|err| err.into())
    }
}

impl Player {
    fn diff_stars(&self, prev: &Player) -> BTreeMap<Day, NewStars> {
        self.completion_day_level
            .iter()
            .fold(BTreeMap::new(), |mut acc, (day, dc)| {
                let new_stars = dc.diff(prev.completion_day_level.get(day));
                // `dc.diff` can return 0, which we don't want to record.
                if new_stars.count() == 1 || new_stars.count() == 2 {
                    acc.insert(*day, new_stars);
                }
                acc
            })
    }

    /// Get the time of the last acquired star
    ///
    /// Currently supplied directly by the API but nice to have it abstracted as a fn call instead
    /// of reading a field. We may want to calculate it locally.
    fn last_star_ts(&self) -> Option<TimeStamp> {
        self.last_star_ts
    }

    fn score(&self) -> Score {
        Score {
            stars: self.stars,
            local: self.local_score,
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
struct PlayerId(u32);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DayCompletion {
    #[serde(rename = "1")]
    star_1: StarProgress,
    #[serde(rename = "2")]
    star_2: Option<StarProgress>,
}

impl DayCompletion {
    fn diff(&self, other: Option<&DayCompletion>) -> NewStars {
        match other {
            // If the key exists in prev, the first star must be taken.
            // Check if the second star is taken in the new data but not in the prev.
            // If yes, return both timestamps since we want to display two stars, even though there is only
            // one new star.
            Some(prev_dc) => {
                if prev_dc.star_2.is_none() && self.star_2.is_some() {
                    NewStars(vec![self.star_1.ts, self.star_2.unwrap().ts])
                } else {
                    NewStars(vec![])
                }
            }
            // If the key does not exist in prev, then either one or both stars have been
            // acquired since prev.
            None => {
                match self.star_2 {
                    None => NewStars(vec![self.star_1.ts]),
                    Some(star_2) => NewStars(vec![self.star_1.ts, star_2.ts]),
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct StarProgress {
    #[serde(rename = "get_star_ts")]
    #[serde(deserialize_with = "de_timestamp")]
    ts: TimeStamp,
}

pub fn get_local_data(file: &str) -> Result<AocData, AocError> {
    let mut file = File::open(file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_json::from_str(&contents).map_err(|err| err.into())
}

pub async fn get_aoc_data() -> Result<AocData, AocError> {
    let client = reqwest::Client::new();
    let aoc_cookie = get_session_cookie()?;
    let res = client
        .get("https://adventofcode.com/2020/leaderboard/private/view/152507.json")
        .header(COOKIE, aoc_cookie)
        .send()
        .await?
        .text()
        .await?;
    Ok(serde_json::from_str(&res)?)
}

fn get_session_cookie() -> Result<String, AocError> {
    let env_var = "AOC_SESSION";
    match env::var(env_var) {
        Ok(token) => Ok(format!("session={}", token)),
        Err(err) => Err(AocError::Env {
            env_var: String::from(env_var),
            source: err,
        }),
    }
}

#[derive(Debug, Error)]
pub enum AocError {
    #[error("Data could not be deserialized")]
    Serde(#[from] serde_json::Error),
    #[error("API error: {}", source.to_string())]
    AocApi {
        #[from]
        source: reqwest::Error,
    },
    // TODO: The data crate should not depend on serenity.
    #[error("Discord error: {}", source.to_string())]
    Discord {
        #[from]
        source: serenity::Error,
    },
    #[error("IO error: {}", source.to_string())]
    IO {
        #[from]
        source: std::io::Error,
    },
    #[error("Environment var: '{}': {}", env_var, source.to_string())]
    Env {
        env_var: String,
        source: env::VarError,
    },
}

/// Special parsing of [`PlayerId`]
///
/// The player id in the `owner_id` field is repr. as an int (JSON Number)
/// whereas in the player ids they are strings.
/// Easiest to accept two match arms.
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
