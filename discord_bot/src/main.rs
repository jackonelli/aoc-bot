use aoc_discord_bot::{update_loop, Responder};
use serenity::{model::id::ChannelId, prelude::*};
use std::env;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = ChannelId(
        env::var("CHANNEL_ID")
            .expect("Expected a token in the environment")
            .parse::<u64>()
            .expect("Could not parse CHANNEL_ID"),
    );

    let mut client = Client::builder(&token)
        .event_handler(Responder)
        .await
        .expect("Err creating client");

    tokio::select! {
        _ = update_loop(&channel_id, &token) => {
            println!("The updater stopped unexpectedly")
        }
        _ = client.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}
