use serde::Deserialize;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Debug, Deserialize)]
struct Quote {
    a: String,
    q: String,
    h: String,
}

#[command]
#[aliases("q")]
pub async fn quote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let argument = args.single::<String>()?;

    let url = match argument.as_str() {
        "r" | "random" => "https://zenquotes.io/api/random".to_string(),
        "t" | "today" => "https://zenquotes.io/api/today".to_string(),
        _ => {
            msg.channel_id.say(&ctx.http, "invalid args").await?;
            return Ok(());
        }
    };
    let res = reqwest::get(url).await?.json::<Vec<Quote>>().await?;

    let _quote: String = format!("{} - **{}**", res[0].q, res[0].a);

    msg.channel_id.say(&ctx.http, _quote).await?;

    Ok(())
}
