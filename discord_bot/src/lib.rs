use anyhow::Result;
use aoc_data::{get_aoc_data, get_local_data, STAR_SYMBOL};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use std::time::Duration;
use tokio::{stream::StreamExt, time::throttle};

const API_DELAY: Duration = Duration::from_secs(901);
const STORED_DATA_FILE: &str = "latest.json";
const SCORE_CMD: &str = "?score";
const START_CMD: &str = "!start";

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_ref() {
            SCORE_CMD => match publish_score(&msg.channel_id, &ctx).await {
                Ok(_) => {}
                Err(err) => println!("{}", err),
            },
            START_CMD => {
                println!("Starting");
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
                let mut item_stream = throttle(API_DELAY, futures::stream::repeat(()));
                loop {
                    item_stream.next().await;
                    match update(&msg.channel_id, &ctx).await {
                        Ok(_) => {}
                        Err(err) => println!("{}", err),
                    }
                }
            }
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
async fn update(channel_id: &ChannelId, ctx: &Context) -> Result<()> {
    println!("Checking for updates");
    let prev = get_local_data("latest.json")?;
    let latest_data = get_aoc_data().await?;
    let diff = latest_data.diff(&prev);
    match diff {
        Some(diff) => {
            channel_id.say(&ctx.http, &diff.fmt()).await?;
            latest_data.write_to_file("latest.json")?;
            Ok(())
        }
        None => Ok(()),
    }
}
