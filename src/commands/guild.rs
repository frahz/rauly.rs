use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fs::File;
use std::io::Read;
use tracing::{error, info};

// Importing the Guild struct from the `guild` module (assuming it's defined in `models` module)
use crate::models::guild::*;

#[command]
#[aliases("gg")]
pub async fn get_guild(ctx: &Context, msg: &Message) -> CommandResult {
    // Logging information about starting the process
    info!("Getting guilds");

    // Opening the config file to read guild information
    let mut file = match File::open("config.json") {
        Ok(f) => f,
        Err(e) => {
            // Error handling if the file couldn't be opened
            error!("Couldn't open file: {}", e);
            return Ok(());
        }
    };

    // Reading the contents of the file into a string
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => info!("Parse successful"),
        Err(e) => {
            // Error handling if reading file contents failed
            error!("Parse failed: {}", e);
            return Ok(());
        }
    };

    // Parsing the JSON contents into the Guilds struct
    let config: Guilds = match serde_json::from_str(&contents) {
        Ok(res) => res,
        Err(e) => {
            // Error handling if parsing JSON failed
            error!("Couldn't read JSON: {}", e);
            return Ok(());
        }
    };

    // Iterating through the guilds and logging their information
    for guild in &config {
        info!("guild: {:#?}", guild);
    }

    // Testing update for the first and second guild in the configuration
    config[1].test_update(&ctx.http).await?;
    config[0].test_update(&ctx.http).await?;

    // Sending a message to the channel indicating successful read of guilds
    msg.channel_id
        .say(&ctx.http, "Read guilds successfully")
        .await?;

    // Returning Ok to signify successful command execution
    Ok(())
}
