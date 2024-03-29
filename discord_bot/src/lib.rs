use crate::config::AocBotConfig;
use anyhow::Result;
use aoc_data::{get_aoc_data, get_local_data, STAR_SYMBOL};
use serenity::{
    async_trait,
    http::Http,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ApplicationId, ChannelId},
    },
    prelude::*,
};
use tokio::time::{interval, Duration};
pub mod config;

const STORED_DATA_FILE: &str = "latest.json";
const SCORE_CMD: &str = "?score";

pub async fn try_responder_client_and_updater_from_config(
    config: AocBotConfig,
) -> Result<(Client, Updater)> {
    let responder = Client::builder(&config.token)
        .application_id(config.application_id.into())
        .event_handler(Responder)
        .await
        .expect("Err creating client");
    let updater = Updater::try_from_config(config)?;
    Ok((responder, updater))
}

pub struct Updater {
    api_delay: Duration,
    token: String,
    application_id: ApplicationId,
    channel_id: ChannelId,
    aoc_cookie: String,
}

impl Updater {
    pub fn try_new(
        api_delay: Duration,
        token: String,
        application_id: ApplicationId,
        channel_id: ChannelId,
        aoc_cookie: String,
    ) -> Result<Self> {
        Ok(Self {
            api_delay,
            token,
            application_id,
            channel_id,
            aoc_cookie,
        })
    }

    pub fn try_from_config(config: AocBotConfig) -> Result<Self> {
        Self::try_new(
            config.api_delay,
            config.token,
            config.application_id,
            config.channel_id,
            config.aoc_cookie,
        )
    }

    /// Respond with current score
    async fn publish_score(channel_id: &ChannelId, ctx: &Context) -> Result<Message> {
        let aoc_data = get_local_data(STORED_DATA_FILE)?;
        let msg = channel_id.say(&ctx.http, &aoc_data.scores_fmt()).await?;
        Ok(msg)
    }

    /// Check for and publish update
    async fn test_update(&self, http: &Http) -> Result<()> {
        println!("Checking for updates");
        self.channel_id.say(http, "Update").await?;
        Ok(())
    }

    /// Periodic update loop
    pub async fn update_loop(&self) -> Result<()> {
        let http =
            &Http::new_with_token_application_id(&self.token, u64::from(self.application_id));
        // Get previously stored data.
        // If not present: Download data from API and store that before entering the loop.
        // Can panic, but it is before entering the loop and if we cannot acquire the
        // initial data then the bot cannot recover.
        match get_local_data(STORED_DATA_FILE) {
            Ok(_) => {}
            Err(_) => {
                let prev = get_aoc_data(&self.aoc_cookie)
                    .await
                    .expect("Could not get initial AocData");
                prev.write_to_file(STORED_DATA_FILE)
                    .expect("Could not write initial data to file");
            }
        };
        let mut interval = interval(self.api_delay);
        loop {
            interval.tick().await;
            match self.update(http).await {
                Ok(_) => {}
                Err(err) => println!("{}", err),
            }
        }
    }

    /// Check for and publish update
    async fn update(&self, http: &Http) -> Result<()> {
        println!("Checking for updates");
        let prev = get_local_data("latest.json")?;
        let latest_data = get_aoc_data(&self.aoc_cookie).await?;
        let diff = latest_data.diff(&prev);
        match diff {
            Some(diff) => {
                self.channel_id.say(http, &diff.fmt()).await?;
                latest_data.write_to_file("latest.json")?;
                Ok(())
            }
            None => Ok(()),
        }
    }
}

/// Bot sub-part actively listening to a channel
struct Responder;

#[async_trait]
impl EventHandler for Responder {
    /// React to messages
    ///
    /// The actual listening is handled with some Serenity magic.
    /// Here, we just match on the message content and react accordingly.
    async fn message(&self, ctx: Context, msg: Message) {
        //println!("Got msg: '{:?}'", &msg);

        // Temp. hack: any time the bot is @:ed, then print score.
        if let Some(mnt) = msg.mentions.get(0) {
            // println!("{:?}", mnt);
            if mnt.name == "aoc-bot" {
                match Updater::publish_score(&msg.channel_id, &ctx).await {
                    Ok(_) => {}
                    Err(err) => println!("{}", err),
                }
            }
        }

        // Old version with '?score' command
        // match msg.mentions[0].name.as_ref() {
        //     SCORE_CMD => match Updater::publish_score(&msg.channel_id, &ctx).await {
        //         Ok(_) => {}
        //         Err(err) => println!("{}", err),
        //     },
        //     // Dummy matched in lieu of a real future command (e.g. "?day_progress")
        //     // My linter complains on an empty match, and who am I to blow against the wind?
        //     "aba" => {}
        //     _ => {}
        // }
    }
    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected! {}", ready.user.name, STAR_SYMBOL);
    }
}
