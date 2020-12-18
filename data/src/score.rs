use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};

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

#[derive(Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct StarCount(pub(crate) u32);

#[derive(Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct LocalScore(u32);

#[derive(Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct GlobalScore(u32);
