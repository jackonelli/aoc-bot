use aoc_bot::aoc_data::{get_local_data, AocData, AocError};

//#[tokio::main]
async fn _main() -> Result<(), AocError> {
    let latest = get_local_data("alatest.json");
    println!("Latest: {}", latest.err().unwrap());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AocError> {
    let latest: AocData = get_local_data("latest.json")?;
    println!("Latest: {:?}", latest.latest_star());
    let prev: AocData = get_local_data("prev.json")?;
    match latest.diff(&prev) {
        Some(diff) => println!("{}", diff.fmt()),
        None => println!("No news"),
    }
    Ok(())
}
