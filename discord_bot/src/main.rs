use aoc_discord_bot::{update_loop, Responder};
use serenity::{model::id::{ApplicationId, ChannelId}, prelude::*};
use std::env;

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a 'DISCORD_TOKEN' in the environment");
    let channel_id = ChannelId(
        env::var("CHANNEL_ID")
            .expect("Expected a 'CHANNEL_ID' in the environment")
            .parse::<u64>()
            .expect("Could not parse CHANNEL_ID"),
    );

    let app_id = ApplicationId(
        env::var("APPLICATION_ID")
            .expect("Expected a 'APPLICATION_ID' in the environment")
            .parse::<u64>()
            .expect("Could not parse APPLICATION_ID"),
    );

    let mut client = Client::builder(&token)
        .application_id(app_id.into())
        .event_handler(Responder)
        .await
        .expect("Err creating client");

    tokio::select! {
        _ = update_loop(&channel_id, &app_id, &token) => {
            println!("The updater stopped unexpectedly")
        }
        _ = client.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}
