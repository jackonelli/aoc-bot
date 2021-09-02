use aoc_discord_bot::{update_loop, Responder, config::AocBotConfig};
use serenity::prelude::*;

#[tokio::main]
async fn main() {
    let config = AocBotConfig::from_config("sample_config.json").expect("Config read failed.");

    let mut client = Client::builder(&config.token)
        .application_id(config.application_id.into())
        .event_handler(Responder)
        .await
        .expect("Err creating client");

    tokio::select! {
        _ = update_loop(&config) => {
            println!("The updater stopped unexpectedly")
        }
        _ = client.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}
