use crate::{Context, Error};
use serde::Deserialize;

#[derive(poise::ChoiceParameter)]
pub enum QuoteChoice {
    #[name = "Today's Quote"]
    Today,
    #[name = "Random Quote"]
    Random,
}

#[derive(Deserialize)]
struct Quote {
    a: String,
    q: String,
}

#[poise::command(slash_command)]
pub async fn quote(ctx: Context<'_>, choice: QuoteChoice) -> Result<(), Error> {
    let url = match choice {
        QuoteChoice::Random => "https://zenquotes.io/api/random",
        QuoteChoice::Today => "https://zenquotes.io/api/today",
    };
    let res = reqwest::get(url).await?.json::<Vec<Quote>>().await?;

    let quote_fmt = format!("{} - **{}**", res[0].q, res[0].a);

    ctx.say(quote_fmt).await?;

    Ok(())
}
