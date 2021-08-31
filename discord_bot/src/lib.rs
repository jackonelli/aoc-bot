use std::sync::Arc;

use anyhow::Result;
use aoc_data::{get_aoc_data, get_local_data, STAR_SYMBOL};
use serenity::{
    async_trait,
    http::Http,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use tokio::time::{interval, Duration};

const API_DELAY: Duration = Duration::from_secs(901);
const STORED_DATA_FILE: &str = "latest.json";
const SCORE_CMD: &str = "?score";

pub struct Responder;

#[async_trait]
impl EventHandler for Responder {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_ref() {
            SCORE_CMD => match publish_score(&msg.channel_id, &ctx).await {
                Ok(_) => {}
                Err(err) => println!("{}", err),
            },
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

/// Periodic update loop
pub async fn update_loop(channel_id: &ChannelId, token: &str) -> Result<()> {
    let http = &Http::new(Arc::new(reqwest::Client::new()), token);
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
