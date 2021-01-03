//! Test Diff impl based on files `time_1.json`, `time_2.json`
//! Originally the same file from the official AoC API with the following manual changes:
//!
//! `time_1.json`:
//!
//! Remove Players:
//! - Alexander, has no stars
//! - Erik, has stars
//! Remove stars, but keep players:
//! - Broms day 16, both, day 4, star 2
//! - Niklas day 17, both, day 1, both
//! - Jern, Day 12, both
//! Modify timestamps
//!
//! `time_2.json`:
//!
//! Remove players:
//! - Jonathan
//! Remove stars:
//! - Jern, Day 12, star 2

use aoc_data::diff::{Diff, NewStars};
use aoc_data::time::{Day, TimeStamp};
use aoc_data::{get_local_data, AocData};
use std::collections::HashMap;
use std::panic;

#[test]
fn check_new_players() {
    run_test(|diff| {
        // Only two new players: Alexander and Erik
        assert!(diff.new_players().count() == 2);
        assert!(
            diff.new_players()
                .map(|pl| &pl.name)
                .filter(|name| *name == "Alexander" || *name == "Erik")
                .count()
                == 2
        );
    })
}

#[test]
fn check_removed_players() {
    run_test(|diff| {
        // Only one removed player: Jonathan
        assert!(diff.removed_players().count() == 1);
        assert!(diff
            .removed_players()
            .map(|pl| &pl.name)
            .any(|name| *name == "Jonathan"));
    })
}

#[test]
fn check_stars() {
    run_test(|diff| {
        let new_stars = diff
            .new_stars()
            .map(|(name, progress)| {
                (
                    name.clone(),
                    progress
                        .clone()
                        .into_iter()
                        .collect::<Vec<(Day, NewStars)>>(),
                )
            })
            .collect::<HashMap<String, Vec<(Day, NewStars)>>>();

        let true_ = vec![
            (
                String::from("Jern"),
                vec![(
                    Day::try_new(12).unwrap(),
                    NewStars::new(vec![TimeStamp::new(1608328926)]),
                )],
            ),
            (
                String::from("Niklas"),
                vec![
                    (
                        Day::try_new(1).unwrap(),
                        NewStars::new(vec![TimeStamp::new(1608492670), TimeStamp::new(1608492671)]),
                    ),
                    (
                        Day::try_new(17).unwrap(),
                        NewStars::new(vec![TimeStamp::new(1608492668), TimeStamp::new(1608492669)]),
                    ),
                ],
            ),
            (
                String::from("Broms"),
                vec![
                    (
                        Day::try_new(4).unwrap(),
                        NewStars::new(vec![TimeStamp::new(1607266996), TimeStamp::new(1608874817)]),
                    ),
                    (
                        Day::try_new(16).unwrap(),
                        NewStars::new(vec![TimeStamp::new(1608874815), TimeStamp::new(1608874816)]),
                    ),
                ],
            ),
        ]
        .into_iter()
        .map(|(name, mut progress)| {
            progress.sort();
            (name, progress)
        })
        .collect::<HashMap<String, Vec<(Day, NewStars)>>>();
        assert!(true_ == new_stars);
    })
}

fn run_test<T>(test: T)
where
    T: FnOnce(&Diff) -> () + panic::UnwindSafe,
{
    let earlier: AocData =
        get_local_data("tests/data/time_1.json").expect("File: 'time_1.json' missing");
    let later: AocData =
        get_local_data("tests/data/time_2.json").expect("File: 'time_2.json' missing");
    let diff = later.diff(&earlier).expect("Diff should be Some");

    let result = panic::catch_unwind(|| test(&diff));

    assert!(result.is_ok())
}
