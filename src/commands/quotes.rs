use serde::Deserialize;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
struct Obj {
    items: Vec<Quote>,
}

#[derive(Debug, Deserialize)]
struct Quote {
    a: String,
    q: String,
    h: String,
}

#[command]
pub async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.single::<f64>()?;
    let two = args.single::<f64>()?;

    let product = one * two;

    msg.channel_id.say(&ctx.http, product).await?;

    Ok(())
}

#[command]
pub async fn today(ctx: &Context, msg: &Message) -> CommandResult {
    let res = reqwest::get("https://zenquotes.io/api/today")
        .await?
        .json::<Obj>()
        .await?;

    let quote: String = format!("{} - **{}**", res.items[0].q, res.items[0].a);

    msg.channel_id.say(&ctx.http, quote).await?;

    Ok(())
}
