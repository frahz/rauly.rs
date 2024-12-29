use crate::{Context, Error};

#[poise::command(prefix_command, aliases("*"))]
pub async fn multiply(ctx: Context<'_>, first: f64, second: f64) -> Result<(), Error> {
    let product = first * second;

    ctx.say(product.to_string()).await?;

    Ok(())
}
