use aoc_bot::aoc_data::{AocData, get_aoc_data, get_latest_local};
use std::fs::File;
use std::io::prelude::*;
use aoc_bot::discord_client::Handler;
use std::env;
use serenity::prelude::*;

//#[tokio::main]
//async fn main() {
//    //let latest_data = get_aoc_data();
//
//    //serde_json::to_writer(&File::create("latest.json").unwrap(), &latest_data.await.unwrap()).unwrap();
//    let file = "latest.json";
//    let mut file = File::open(file).expect("Opening file error");
//    let mut contents = String::new();
//    file.read_to_string(&mut contents)
//        .expect("Read to string error");
//    let latest_data: AocData = serde_json::from_str(&contents).expect("parse_error");
//    println!("{}", latest_data.scores_fmt());
//    println!("{:?}", latest_data.latest_star());
//}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
