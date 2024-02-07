use anyhow::Result;
use chrono::prelude::*;
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serenity::http::Http;
use serenity::model::prelude::*;
use tracing::info;

pub type Guilds = Vec<Guild>;
#[derive(Debug, Serialize, Deserialize)]
pub struct Guild {
    setup: bool,
    guild: String,
    guild_id: GuildId,
    wotd_channel: String,
    wotd_channel_id: ChannelId,
    timezone: String,
    wotd_time: String,
}

impl Guild {
    // pub fn from_config(config: &str) -> Result<Guild>{
    //     let mut file = File::open(config)?;
    //     let mut content = String::new();
    //     file.read_to_string(&mut content)?;
    //     Ok(serde_json::from_str(&content)?)
    // }
    pub async fn test_update(&self, http: &Http) -> Result<Message> {
        info!("Sending update");
        let tz = self.timezone.parse::<Tz>().unwrap();
        info!("{}", self.wotd_time);
        let wotime = NaiveTime::parse_from_str(&self.wotd_time, "%H:%M").unwrap();
        let time = Utc::now().with_timezone(&tz);
        let msg = self
            .wotd_channel_id
            .say(
                &http,
                format!("Testing Now: {}\n Word of the Day Time: {}", time, wotime),
            )
            .await?;
        Ok(msg)
    }
}
