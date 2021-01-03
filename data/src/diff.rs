use crate::time::{Day, TimeStamp};
use crate::{Player, STAR_SYMBOL};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub struct Diff {
    pub(crate) new_players: Vec<Player>,
    pub(crate) removed_players: Vec<Player>,
    pub(crate) new_stars: HashMap<String, BTreeMap<Day, NewStars>>,
}

impl Diff {
    pub fn fmt(self) -> String {
        let mut fmt_diff = String::new();
        for (pl, stars) in self.new_stars.iter() {
            for (day, new_stars) in stars {
                fmt_diff.push_str(&format!(
                    "{0: <20} - Dag {1: <2}: {2:<2}\n",
                    pl,
                    day,
                    new_stars.fmt()
                ));
            }
        }
        if !self.new_players.is_empty() {
            fmt_diff.push_str("New players: ");
            for pl in &self.new_players {
                fmt_diff.push_str(&format!("{} ", pl.name));
            }
        }
        fmt_diff
    }

    pub fn new_players(&self) -> impl Iterator<Item = &Player> {
        self.new_players.iter()
    }

    pub fn removed_players(&self) -> impl Iterator<Item = &Player> {
        self.removed_players.iter()
    }

    pub fn new_stars(&self) -> impl Iterator<Item = (&String, &BTreeMap<Day, NewStars>)> {
        self.new_stars.iter()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NewStars(pub(crate) Vec<TimeStamp>);

impl NewStars {
    /// Format new stars for update
    ///
    /// 1 or 2 star emojis, with corresponding timestamp in hour and minue resolution
    fn fmt(&self) -> String {
        let mut str_ = String::new();
        str_.push_str(&STAR_SYMBOL.to_string().repeat(self.0.len()));
        let times = if self.0.len() == 1 {
            format!(" ({})", self.0[0])
        } else {
            format!(" ({}, {})", self.0[0], self.0[1])
        };
        str_.push_str(&times);
        str_
    }

    pub fn new(tss: Vec<TimeStamp>) -> Self {
        NewStars(tss)
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }
}
