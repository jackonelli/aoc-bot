use anyhow::Result;
use aoc_data::{get_aoc_data, get_local_data, STAR_SYMBOL};
use serenity::{async_trait, http::Http, model::{channel::Message, gateway::Ready, id::{ChannelId}}, prelude::*};
use tokio::time::{interval, Duration};
use crate::config::AocBotConfig;
pub mod config;

// const API_DELAY: Duration = Duration::from_secs(901);
const API_DELAY: Duration = Duration::from_secs(5);
const STORED_DATA_FILE: &str = "latest.json";
const SCORE_CMD: &str = "?score";

/// Bot sub-part actively listening to a channel
pub struct Responder;

#[async_trait]
impl EventHandler for Responder {
    /// React to messages
    ///
    /// The actual listening is handled with some Serenity magic.
    /// Here, we just match on the message content and react accordingly.
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_ref() {
            SCORE_CMD => match publish_score(&msg.channel_id, &ctx).await {
                Ok(_) => {}
                Err(err) => println!("{}", err),
            },
            // Dummy matched in lieu of a real future command (e.g. "?day_progress")
            // My linter complains on an empty match, and who am I to blow against the wind?
            "aba" => {},
            _ => {}
        }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected! {}", ready.user.name, STAR_SYMBOL);
    }
}

/// Respond with current score
async fn publish_score(channel_id: &ChannelId, ctx: &Context) -> Result<Message> {
    let aoc_data = get_local_data(STORED_DATA_FILE)?;
    let msg = channel_id.say(&ctx.http, &aoc_data.scores_fmt()).await?;
    Ok(msg)
}

/// Check for and publish update
async fn test_update(channel_id: &ChannelId, http: &Http) -> Result<()> {
    println!("Checking for updates");
    channel_id.say(http, "Update").await?;
    Ok(())
}

/// Periodic update loop
pub async fn update_loop(config: &AocBotConfig) -> Result<()> {
    let (token, channel_id, app_id) = (&config.token, &config.channel_id, &config.application_id);
    let http = &Http::new_with_token_application_id(token, u64::from(*app_id));
    // Get previously stored data.
    // If not present: Download data from API and store that before entering the loop.
    // Can panic, but it is before entering the loop and if we cannot acquire the
    // initial data then the bot cannot recover.
    match get_local_data(STORED_DATA_FILE) {
        Ok(_) => {}
        Err(_) => {
            let prev = get_aoc_data().await.expect("Could not get initial AocData");
            prev.write_to_file(STORED_DATA_FILE)
                .expect("Could not write initial data to file");
        }
    };
    let mut interval = interval(API_DELAY);
    loop {
        interval.tick().await;
        match update(channel_id, http).await {
            Ok(_) => {}
            Err(err) => println!("{}", err),
        }
    }
}

/// Check for and publish update
async fn update(channel_id: &ChannelId, http: &Http) -> Result<()> {
    println!("Checking for updates");
    let prev = get_local_data("latest.json")?;
    let latest_data = get_aoc_data().await?;
    let diff = latest_data.diff(&prev);
    match diff {
        Some(diff) => {
            channel_id.say(http, &diff.fmt()).await?;
            latest_data.write_to_file("latest.json")?;
            Ok(())
        }
        None => Ok(()),
    }
}
