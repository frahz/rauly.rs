use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[aliases("*")]
pub async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // Attempt to parse the first argument as a f64
    let one = args.single::<f64>()?;

    // Attempt to parse the second argument as a f64
    let two = args.single::<f64>()?;

    // Calculate the product of the two parsed numbers
    let product = one * two;

    // Send the calculated product as a message to the channel
    msg.channel_id.say(&ctx.http, product).await?;

    // Return Ok to indicate successful command execution
    Ok(())
}
