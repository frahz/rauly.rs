use crate::{models::word, utils};
use chrono::prelude::*;
use rand::prelude::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn word(ctx: &Context, msg: &Message) -> CommandResult {
    let dt = Utc::now().format("%B %d, %Y");
    let color = utils::COLORS.choose(&mut rand::thread_rng()).unwrap();

    let res = match word::word().await {
        Ok(r) => r,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "had a problem parsing JSON!")
                .await?;
            return Ok(());
        }
    };

    let example = res.examples[0]
        .text
        .replace(&res.word, &format!("**{}**", res.word));

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} | {}", res.word, dt))
                    .colour(*color)
                    // .field("Pronounciation", res.publish_date, false)
                    .field(
                        "Word type",
                        format!("*{}*", res.definitions[0].part_of_speech),
                        false,
                    )
                    .field("Definition", &res.definitions[0].text, false)
                    .field("Example", example, false)
                    .field("Note", res.note, false)
                    .footer(|f| {
                        f.text("Word of the Day");
                        f
                    })
            })
        })
        .await?;

    Ok(())
}
