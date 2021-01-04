//! Test local 'local score' impl
//!
//! The file `time_2.json` contains data for the 2020 comptetition where the local score for day 1
//! was ignored

use aoc_data::score::LocalScore;
use aoc_data::{get_local_data, AocData, PlayerId};
use std::collections::HashMap;
use std::panic;

#[test]
fn check_local_scores() {
    run_test(|data| {
        let mut local_ls: Vec<(PlayerId, LocalScore)> = data
            .local_scores()
            .iter()
            .map(|(id, day_scores)| {
                (
                    id.clone(),
                    day_scores.iter().map(|(_, day_scores)| day_scores).sum(),
                )
            })
            .collect();
        local_ls.sort_by_key(|(_, ls)| *ls);
        let mut remote_ls: Vec<(PlayerId, LocalScore)> = data
            .players()
            .map(|(id, pl)| (id.clone(), pl.local_score))
            .collect();
        remote_ls.sort_by_key(|(_, ls)| *ls);

        assert!(local_ls == remote_ls);
    })
}

fn run_test<T>(test: T)
where
    T: FnOnce(&AocData) + panic::UnwindSafe,
{
    let data: AocData =
        get_local_data("tests/data/time_2.json").expect("File: 'time_2.json' missing");

    let result = panic::catch_unwind(|| test(&data));

    assert!(result.is_ok())
}
