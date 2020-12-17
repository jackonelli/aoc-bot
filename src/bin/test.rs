use aoc_bot::aoc_data::{get_local_data, AocData, AocError};

fn main() -> Result<(), AocError> {
    let latest: AocData = get_local_data("latest.json")?;
    println!("Latest: {:?}", latest.latest_star());
    let prev: AocData = get_local_data("prev.json")?;
    match latest.diff(&prev) {
        Some(diff) => println!("{}", diff.fmt()),
        None => println!("No news"),
    }
    Ok(())
}
