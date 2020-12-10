use aoc_bot::aoc_data::get_aoc_data;

#[tokio::main]
async fn main() {
    let latest_data = get_aoc_data();
    println!("{:?}", latest_data.await);
}
