use aoc_bot::aoc_data::{get_local_data, AocData};

#[tokio::main]
async fn main() {
    let latest: AocData = get_local_data("latest.json");
    let prev: AocData = get_local_data("prev.json");
    println!("{:?}", latest.latest_star());
    match latest.diff(&prev) {
        Some(diff) => println!("{}", diff.fmt()),
        None => println!("No news")
    }
}
