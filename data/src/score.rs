use derive_more::{Add, Display};
use num::Zero;
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::iter::Sum;

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

#[derive(
    Add, Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct LocalScore(pub(crate) u32);

impl Zero for LocalScore {
    fn zero() -> Self {
        LocalScore(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Sum<Self> for LocalScore {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Self>,
    {
        iter.fold(Self(0), |a, b| Self(a.0 + b.0))
    }
}

#[derive(
    Add, Copy, Clone, Debug, Display, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq,
)]
pub struct GlobalScore(pub(crate) u32);
impl Zero for GlobalScore {
    fn zero() -> Self {
        GlobalScore(0)
    }
    fn is_zero(&self) -> bool {
        self.0 == 0
    }
}
