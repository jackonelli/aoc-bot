use aoc_data::{get_local_data, AocData, AocError};

fn main() -> Result<(), AocError> {
    let latest: AocData = get_local_data("data/tests/data/time_2.json")?;
    let prev: AocData = get_local_data("data/tests/data/time_1.json")?;
    match latest.diff(&prev) {
        Some(diff) => println!("{}", diff.fmt()),
        None => println!("No news"),
    }

    Ok(())
}
