use aoc_data::{get_local_data, AocData, AocError};

fn main() -> Result<(), AocError> {
    let latest: AocData = get_local_data("latest_debug.json")?;
    println!("Latest: {:?}", latest.latest_star());
    let prev: AocData = get_local_data("prev_debug.json")?;
    println!("{:?}", prev.latest_star().unwrap().hour_and_minute());
    match latest.diff(&prev) {
        Some(diff) => println!("{}", diff.fmt()),
        None => println!("No news"),
    }
    Ok(())
}
