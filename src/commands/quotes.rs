use serde::Deserialize;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

// Deserialization structure for the quote data
#[derive(Debug, Deserialize)]
struct Quote {
    a: String, // Author of the quote
    q: String, // Quote itself
    h: String, // Some other field not being used here
}

// Command for retrieving quotes
#[command]
#[aliases("q")]
pub async fn quote(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Retrieve the argument provided with the command
    let argument = args.single::<String>()?;

    // Define different URLs based on the provided argument
    let url = match argument.as_str() {
        "r" | "random" => "https://zenquotes.io/api/random".to_string(), // Random quote URL
        "t" | "today" => "https://zenquotes.io/api/today".to_string(),   // Today's quote URL
        _ => {
            // If the argument doesn't match "r", "random", "t", or "today", inform about invalid argument
            msg.channel_id.say(&ctx.http, "invalid args").await?;
            return Ok(());
        }
    };

    // Make a request to the API and deserialize the response into a vector of Quote structs
    let res = reqwest::get(url).await?.json::<Vec<Quote>>().await?;

    // Format the quote and author into a string
    let _quote: String = format!("{} - **{}**", res[0].q, res[0].a);

    // Send the formatted quote to the channel where the command was invoked
    msg.channel_id.say(&ctx.http, _quote).await?;

    // Return Ok to indicate successful command execution
    Ok(())
}
