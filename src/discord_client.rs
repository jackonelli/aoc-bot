use std::env;
use std::time::Duration;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

const API_DELAY: Duration = Duration::from_secs(5);
const HELP_CMD: &str = "?help";
const START_CMD: &str = "!start";
const STAR_EMOJI: char = '\u{2B50}';

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == HELP_CMD {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Here's help").await {
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


//#[tokio::main]
//async fn main() {
//    let token = env::var("DISCORD_TOKEN")
//        .expect("Expected a token in the environment");
//
//    let mut client = Client::builder(&token)
//        .event_handler(Handler)
//        .await
//        .expect("Err creating client");
//
//    if let Err(why) = client.start().await {
//        println!("Client error: {:?}", why);
//    }
//}
