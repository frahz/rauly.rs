use anyhow::Result; // Importing the Result type from the `anyhow` crate
use chrono::prelude::*; // Importing date and time related functionalities from `chrono`
use chrono_tz::Tz; // Importing time zone functionalities from `chrono_tz`
use serde::{Deserialize, Serialize}; // Importing traits for serialization and deserialization
use serenity::http::Http; // Importing HTTP client from Serenity
use serenity::model::prelude::*; // Importing Serenity's model functionalities
use tracing::{error, info}; // Importing logging functionalities

pub type Guilds = Vec<Guild>; // Alias for a vector of Guild structs

#[derive(Debug, Serialize, Deserialize)]
pub struct Guild {
    setup: bool,                // Indicates if the guild has been set up
    guild: String,              // Guild name
    guild_id: GuildId,          // Guild ID
    wotd_channel: String,       // Name of the Word of the Day channel
    wotd_channel_id: ChannelId, // ID of the Word of the Day channel
    timezone: String,           // Timezone of the guild
    wotd_time: String,          // Time for the Word of the Day
}
impl Guild {
    // Method to test and update guild settings
    pub async fn test_update(&self, http: &Http) -> Result<Message> {
        info!("Sending update");

        // Parse the guild's timezone into a chrono_tz::Tz object
        let tz = self.timezone.parse::<Tz>().unwrap();

        // Parse the Word of the Day time into a chrono::NaiveTime object
        let wotime = NaiveTime::parse_from_str(&self.wotd_time, "%H:%M").unwrap();

        // Get the current time in the guild's timezone
        let time = Utc::now().with_timezone(&tz);

        // Send a test message to the Word of the Day channel
        let msg = self
            .wotd_channel_id
            .say(
                &http,
                format!("Testing Now: {}\n Word of the Day Time: {}", time, wotime),
            )
            .await?;

        Ok(msg) // Return the sent message
    }
}
