use crate::{Context, Error};

// #[poise::command(prefix_command, aliases("*"))]
#[poise::command(slash_command)]
pub async fn multiply(
    ctx: Context<'_>,
    #[description = "First number"] first: f64,
    #[description = "Second Number"] second: f64,
) -> Result<(), Error> {
    let product = first * second;

    ctx.say(product.to_string()).await?;

    Ok(())
}
