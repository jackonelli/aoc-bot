use aoc_data::{get_aoc_data, get_local_data, AocData, AocError, STAR_EMOJI};
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use std::time::Duration;

const API_DELAY: Duration = Duration::from_secs(901);
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
                loop {
                    match update(&msg.channel_id, &ctx).await {
                        Ok(_) => {}
                        Err(err) => println!("{}", err),
                    }
                    // TODO: look up throttle.
                    tokio::time::delay_for(API_DELAY).await;
                }
            }
            _ => {}
        }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected! {}", ready.user.name, STAR_EMOJI);
    }
}

/// Respond with current score
async fn publish_score(channel_id: &ChannelId, ctx: &Context) -> Result<Message, AocError> {
    let aoc_data = get_local_data("latest.json")?;
    channel_id
        .say(&ctx.http, &aoc_data.scores_fmt())
        .await
        .map_err(|err| AocError::Discord { source: err })
}

/// Check for and publish update
async fn update(channel_id: &ChannelId, ctx: &Context) -> Result<(), AocError> {
    println!("Checking for updates");
    let latest_data = get_aoc_data().await?;
    let prev: AocData = get_local_data("latest.json")?;
    let diff = latest_data.diff(&prev);
    match diff {
        Some(diff) => {
            channel_id
                .say(&ctx.http, &diff.fmt())
                .await
                .map_err(|err| AocError::Discord { source: err })?;
            latest_data.write_to_file("latest.json")
        }
        None => Ok(()),
    }
}
