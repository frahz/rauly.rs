use chrono::prelude::*;
use rand::seq::SliceRandom;
use std::env;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serde::{Serialize, Deserialize};
use tracing::{error, info};
use crate::utils;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Response {
    #[serde(rename = "_id")]
    id: String,
    word: String,
    content_provider: ContentProvider,
    definitions: Vec<Definition>,
    publish_date: String,
    examples: Vec<Example>,
    pdd: String,
    html_extra: Option<String>,
    note: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContentProvider {
    name: String,
    id: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Definition {
    source: String,
    text: String,
    note: Option<String>,
    part_of_speech: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Example {
    url: String,
    title: String,
    text: String,
    id: Option<u32>,
}

#[command]
pub async fn word(ctx: &Context, msg: &Message) -> CommandResult {
    info!("Sending Word of the Day");
    let wordnik_api = env::var("WORDNIK_API_KEY").expect("wordnik api key");
    let url = format!("https://api.wordnik.com/v4/words.json/wordOfTheDay?api_key={}",wordnik_api);

    let dt = Utc::now().format("%B %d, %Y");
    let color = utils::COLORS.choose(&mut rand::thread_rng()).unwrap();

    let res = match reqwest::get(url).await?.json::<Response>().await {
        Ok(r) => r,
        Err(e) => {
            error!("Problem parsing JSON: {:?}", e);
            msg.channel_id.say(&ctx.http, "had a problem parsing JSON!").await?;
            return Ok(());
        },
    };

    let example = res.examples[0].text.replace(&res.word, &format!("**{}**", res.word));

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} | {}", res.word, dt))
                    .colour(*color)
                    // .field("Pronounciation", res.publish_date, false)
                    .field("Word type", format!("*{}*", res.definitions[0].part_of_speech), false)
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

