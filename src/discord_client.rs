use crate::aoc_data::{AocData, get_aoc_data, get_latest_local};
use std::time::Duration;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const API_DELAY: Duration = Duration::from_secs(5);
const SCORE_CMD: &str = "?score";
const START_CMD: &str = "!start";
const STAR_EMOJI: char = '\u{2B50}';

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == SCORE_CMD {
            let aoc_data = get_latest_local();

            if let Err(why) = msg.channel_id.say(&ctx.http, &aoc_data.scores_fmt()).await {
                println!("Error sending message: {:?}", why);
            };
        };
        if msg.content == START_CMD {
            loop {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Update").await {
                    println!("Error sending message: {:?}", why);
                };
                tokio::time::delay_for(API_DELAY).await;
            }
        }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected! {}", ready.user.name, STAR_EMOJI);
    }

}

