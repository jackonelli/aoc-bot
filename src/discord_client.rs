use crate::aoc_data::{get_aoc_data, get_local_data, AocData};
use crate::STAR_EMOJI;
use std::fs::File;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
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
        if msg.content == SCORE_CMD {
            let aoc_data = get_local_data("latest.json");
            if let Err(why) = msg.channel_id.say(&ctx.http, &aoc_data.scores_fmt()).await {
                println!("Error sending message: {:?}", why);
            };
        };
        if msg.content == START_CMD {
            println!("Starting");
            loop {
                println!("Checking for updates");
                let latest_data = get_aoc_data().await.unwrap();
                let prev: AocData = get_local_data("latest.json");
                let diff = latest_data.diff(&prev);
                if diff.is_some() {
                    if let Err(why) = msg.channel_id.say(&ctx.http, &diff.unwrap().fmt()).await {
                        println!("Error sending message: {:?}", why);
                    };
                    serde_json::to_writer(&File::create("latest.json").unwrap(), &latest_data).unwrap();
                }
                tokio::time::delay_for(API_DELAY).await;
            }
        }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected! {}", ready.user.name, STAR_EMOJI);
    }
}
