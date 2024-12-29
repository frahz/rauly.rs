use crate::models::guild::*;
use crate::{Context, Error};
use std::fs::File;
use std::io::Read;
use tracing::{error, info};

#[poise::command(prefix_command, aliases("gg"))]
pub async fn get_guild(ctx: Context<'_>) -> Result<(), Error> {
    info!("Getting guilds");
    let mut file = match File::open("config.json") {
        Ok(f) => f,
        Err(e) => {
            error!("Couldn't open file: {}", e);
            return Ok(());
        }
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => info!("Parse successful"),
        Err(e) => {
            error!("Parse failed: {}", e);
            return Ok(());
        }
    };
    let config: Guilds = match serde_json::from_str(&contents) {
        Ok(res) => res,
        Err(e) => {
            error!("Couldn't read JSON: {}", e);
            return Ok(());
        }
    };
    for guild in &config {
        info!("guild: {:#?}", guild);
    }
    config[0].test_update(ctx.http()).await?;
    config[1].test_update(ctx.http()).await?;
    ctx.say("Read Guilds successfully").await?;
    Ok(())
}
