//! Test local 'local score' impl
//!
//! The file `time_2.json` contains data for the 2020 comptetition where the local score for day 1
//! was ignored

use aoc_data::score::LocalScore;
use aoc_data::time::Day;
use aoc_data::{get_local_data, AocData, PlayerId};
use std::cmp::Reverse;
use std::panic;

#[test]
fn check_local_scores() {
    run_test(|data| {
        let mut local_ls: Vec<(PlayerId, LocalScore)> = data
            .local_scores()
            .iter()
            .map(|(id, day_scores)| {
                (
                    *id,
                    day_scores
                        .iter()
                        .filter(|(day, _)| **day != Day::try_new(1).unwrap())
                        .map(|(_, day_score)| day_score.0 + day_score.1)
                        .sum(),
                )
            })
            .collect();
        local_ls.sort_by_key(|(_, ls)| Reverse(*ls));
        let mut remote_ls: Vec<(PlayerId, LocalScore)> = data
            .players()
            .map(|(id, pl)| (*id, pl.local_score))
            .collect();
        remote_ls.sort_by_key(|(_, ls)| Reverse(*ls));

        //assert!(remote_ls.len() == local_ls.len());

        for ((lpl, lls), (tpl, tls)) in local_ls.iter().zip(remote_ls.iter()) {
            // assert!(lpl == tpl);
            println!(
                "{:?}: {} / {}",
                data.players().find(|(id, _)| *id == lpl).unwrap().1.name,
                lls,
                tls
            );
        }

        assert!(local_ls == remote_ls);
    })
}

fn run_test<T>(test: T)
where
    T: FnOnce(&AocData) + panic::UnwindSafe,
{
    let data: AocData =
        get_local_data("tests/data/test_data.json").expect("File: 'test_data.json' missing");

    let result = panic::catch_unwind(|| test(&data));

    assert!(result.is_ok())
}
