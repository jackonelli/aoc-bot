use aoc_discord_bot::{config::AocBotConfig, try_responder_client_and_updater_from_config};

#[tokio::main]
async fn main() {
    let config = AocBotConfig::from_config("config.json").expect("Parsing `config.json` failed.");

    let (mut responder, updater) = try_responder_client_and_updater_from_config(config).await.expect("Create bot failed.");

    tokio::select! {
        _ = updater.update_loop() => {
            println!("The updater stopped unexpectedly")
        }
        _ = responder.start() => {
            println!("The responder stopped unexpectedly")
        }
    };
}
