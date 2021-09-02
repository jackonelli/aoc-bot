use aoc_discord_bot::{AocBot, config::AocBotConfig};

#[tokio::main]
async fn main() {
    let config = AocBotConfig::from_config("config.json").expect("Config read failed.");

    let mut bot = AocBot::try_from_config(config.clone()).await.expect("Create bot failed.");

    tokio::select! {
        _ = bot.update_loop() => {
            println!("The updater stopped unexpectedly")
        }
        _ = bot.responder.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}
