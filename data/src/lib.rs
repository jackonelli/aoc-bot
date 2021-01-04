//! # Advent of Code data
//!
//! Provides a strictly typed data schema and logic for the [Advent of Code](https://adventofcode.com/) competition API.
pub mod diff;
pub mod score;
pub mod time;
use crate::diff::{Diff, NewStars};
use crate::score::{GlobalScore, LocalScore, Score, StarCount};
use crate::time::{de_opt_timestamp, de_timestamp, sort_optional_ts, Day, TimeStamp};
use itertools::Itertools;
use reqwest::header::COOKIE;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::cmp::{Eq, PartialEq, Reverse};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::convert::TryFrom;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::iter::once;
use thiserror::Error;

/// For nice formatting
pub const STAR_SYMBOL: char = '\u{2B50}';

/// Representation of private leaderboard data return from the AoC API
#[derive(Clone, Debug, Deserialize, Serialize)]
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
pub struct Player {
    /// Display name
    pub name: String,
    /// Progress for each day
    completion_day_level: BTreeMap<Day, DayCompletion>,
    pub local_score: LocalScore,
    pub global_score: GlobalScore,
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

    pub fn local_scores(&self) -> HashMap<PlayerId, BTreeMap<Day, (LocalScore, LocalScore)>> {
        // Create an iterator over stars with corresponding player_id and timestamp (Option<Timestamp>).
        // Uses the tuple (day, star number) as key for the stars.
        // I.e. the result is a tuple of tuples:
        // ( (day, star number), (player_id, optional timestamp) )
        let day_compl = self
            .players()
            .map(|(id, player)| {
                player
                    .completion_day_level
                    .iter()
                    .map(move |(day, day_compl)| (day, id, day_compl))
            })
            .flatten()
            // Split the two stars in a DayCompletion into two separate entries:
            // Group them into tuple of tuples for the later grouping by (key, val)
            .map(|(day, id, day_compl)| {
                once(((day, 1), (id, Some(day_compl.star_1.ts)))).chain(once((
                    (day, 2),
                    (id, day_compl.star_2.map(|star_progress| star_progress.ts)),
                )))
            })
            .flatten()
            // Cool efficient grouping
            .into_grouping_map();

        // Create a hash map with the new 'star_key' groups
        let mut day_map = day_compl.fold(Vec::new(), |mut acc, _key, val| {
            acc.push(val);
            acc
        });

        // Sort vectors of Option<Timestamp>
        day_map.iter_mut().for_each(|(_day_key, player_ts)| {
            player_ts.sort_unstable_by(|(_id_a, ts_a), (_id_b, ts_b)| sort_optional_ts(ts_a, ts_b))
        });

        // Do a similar flattening then grouping to create a map with player id as the key and a
        // btree map with (day, (ls, ls)) as the value.
        let player_day_map = day_map
            .iter()
            .map(|(star_key, player_ts)| {
                player_ts.iter().enumerate().map(move |(idx, (id, ts))| {
                    let factor = ts.map(|_| 1).unwrap_or(0);
                    (
                        **id,
                        (
                            *star_key,
                            LocalScore(
                                u32::try_from(factor * (self.num_players() - idx))
                                    .expect("u32 overflow"),
                            ),
                        ),
                    )
                })
            })
            .flatten()
            .into_grouping_map()
            .fold(BTreeMap::new(), |mut acc, _key, val| {
                let (star_key, ls) = val;
                //merge the individual star_key's into a tuple for each day.
                let (day, star_num) = star_key;
                let score = acc.entry(*day).or_insert((LocalScore(0), LocalScore(0)));
                match star_num {
                    1 => score.0 = ls,
                    2 => score.1 = ls,
                    _ => panic!("Wrong star num"),
                };
                acc
            });

        // Ugly cloning... Don't know how to fix.
        let no_star_players = self
            .player_ids()
            .collect::<HashSet<&PlayerId>>()
            .difference(&player_day_map.keys().collect())
            .map(|id| **id)
            .collect::<Vec<PlayerId>>();
        player_day_map
            .into_iter()
            .chain(no_star_players.into_iter().map(|id| (id, BTreeMap::new())))
            .collect()
    }

    pub fn players(&self) -> impl Iterator<Item = (&PlayerId, &Player)> {
        self.players.iter()
    }

    pub fn num_players(&self) -> usize {
        self.players.len()
    }

    fn player_ids(&self) -> impl Iterator<Item = &PlayerId> {
        self.players.keys()
    }

    fn player_id_set(&self) -> HashSet<&PlayerId> {
        self.player_ids().collect()
    }

    /// Aggregate possible diff
    ///
    /// Compare `self` to a previous data point `prev`.
    /// If there are more recent stars or a change of players, return `Some([Diff])`.
    /// Otherwise return `None`.
    ///
    /// Never panics: unwrapping the access of `self.players[id]` is fine since `id` is in the
    /// set of `new_players` which is a subset of `self.players`.
    /// Same goes for `prev.players[id]` since `id` is in a subset of `prev.players`.
    pub fn diff(&self, prev: &AocData) -> Option<Diff> {
        if self.latest_star() == prev.latest_star() && self.player_id_set() == prev.player_id_set()
        {
            None
        } else {
            let new_players = self
                .player_id_set()
                .difference(&prev.player_id_set())
                .cloned()
                .collect();
            let removed_players = prev
                .player_id_set()
                .difference(&self.player_id_set())
                .map(|id| prev.players.get(&id).unwrap().clone())
                .collect();
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
                removed_players,
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

    pub fn write_to_file(&self, file: &str) -> Result<(), AocError> {
        serde_json::to_writer_pretty(&File::create(file)?, self).map_err(|err| err.into())
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

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct PlayerId(u32);

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
            None => match self.star_2 {
                None => NewStars(vec![self.star_1.ts]),
                Some(star_2) => NewStars(vec![self.star_1.ts, star_2.ts]),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct StarProgress {
    #[serde(rename = "get_star_ts")]
    #[serde(deserialize_with = "de_timestamp")]
    pub(crate) ts: TimeStamp,
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
    #[error("Param {param}:{val} incorrect. {reason}")]
    Param {
        param: String,
        val: String,
        reason: String,
    },
    #[error("Data could not be deserialized")]
    Serde(#[from] serde_json::Error),
    #[error("API error: {}", source.to_string())]
    AocApi {
        #[from]
        source: reqwest::Error,
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

#[cfg(test)]
mod test {
    use super::*;
    use num::Zero;

    /// Specific bug in which new data with no new stars but a new player generated an empty
    /// message. It was likely fixed when 'new players' were added to the formatting but test is
    /// added as a persistent check.
    #[test]
    fn test_only_new_players() {
        let mut players = HashMap::new();
        players.insert(
            PlayerId(0),
            Player {
                name: "Aba".to_string(),
                completion_day_level: BTreeMap::new(),
                local_score: LocalScore::zero(),
                global_score: GlobalScore::zero(),
                stars: StarCount(0),
                last_star_ts: None,
            },
        );
        let prev = AocData {
            event: "Test".to_string(),
            owner_id: PlayerId(0),
            players: players.clone(),
        };
        players.insert(
            PlayerId(1),
            Player {
                name: "Bab".to_string(),
                completion_day_level: BTreeMap::new(),
                local_score: LocalScore::zero(),
                global_score: GlobalScore::zero(),
                stars: StarCount(0),
                last_star_ts: None,
            },
        );
        let later = AocData {
            event: "Test".to_string(),
            owner_id: PlayerId(0),
            players,
        };
        assert!(later.diff(&prev).unwrap().new_players().count() == 1);
        assert!(!later.diff(&prev).unwrap().fmt().is_empty());
    }
}
