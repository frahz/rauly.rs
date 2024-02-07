use crate::{models::word, utils};
use chrono::prelude::*;
use rand::prelude::*;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn word(ctx: &Context, msg: &Message) -> CommandResult {
    let dt = Utc::now().format("%B %d, %Y");
    let color = utils::COLORS.choose(&mut rand::thread_rng()).unwrap();

    let Ok(res) = word::get_word().await else {
        msg.channel_id
            .say(&ctx.http, "had a problem parsing JSON!")
            .await?;
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
    let builder = CreateMessage::new().embed(embed);
    msg.channel_id.send_message(&ctx.http, builder).await?;

    Ok(())
}
