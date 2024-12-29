use crate::{Context, Error};
use serde::Deserialize;

#[derive(Deserialize)]
struct Quote {
    a: String,
    q: String,
}

#[poise::command(prefix_command, aliases("q"))]
pub async fn quote(ctx: Context<'_>, choice: String) -> Result<(), Error> {
    let url = match choice.as_str() {
        "r" | "random" => "https://zenquotes.io/api/random",
        "t" | "today" => "https://zenquotes.io/api/today",
        s => {
            ctx.say(format!(
                "**Invalid Argument Passed**: {}\nPlease pass one of `today` or `random`",
                s
            ))
            .await?;
            return Ok(());
        }
    };
    let res = reqwest::get(url).await?.json::<Vec<Quote>>().await?;

    let quote_fmt = format!("{} - **{}**", res[0].q, res[0].a);

    ctx.say(quote_fmt).await?;

    Ok(())
}
