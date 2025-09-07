use crate::{models::word, utils, Context, Error};
use chrono::prelude::*;
use rand::prelude::*;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};

#[poise::command(
    slash_command,
    description_localized("en-US", "Displays the Word of the Day")
)]
pub async fn word(ctx: Context<'_>) -> Result<(), Error> {
    let dt = Utc::now().format("%B %d, %Y");
    let color = utils::COLORS.choose(&mut rand::thread_rng()).unwrap();

    let Ok(res) = word::get_word().await else {
        ctx.say("Had a problem parsing JSON!").await?;
        return Ok(());
    };

    let example = res.examples[0]
        .text
        .replace(&res.word, &format!("**{}**", res.word));

    let footer = CreateEmbedFooter::new("Word of the Day");
    let embed = CreateEmbed::new()
        .title(format!("{} | {}", res.word, dt))
        .color(*color)
        .field(
            "Word type",
            format!("*{}*", res.definitions[0].part_of_speech),
            false,
        )
        .field("Definition", &res.definitions[0].text, false)
        .field("Example", example, false)
        .field("Note", res.note, false)
        .footer(footer);
    let msg = poise::CreateReply::default().embed(embed);
    ctx.send(msg).await?;

    Ok(())
}
