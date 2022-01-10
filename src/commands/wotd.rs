// use serde::Deserialize;
use chrono::prelude::*;
use rand::seq::SliceRandom;
use scraper::{Html, Selector};
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Word {
    word: String,
    pronounciation: String,
    word_type: String,
    definition: String,
    example: String,
}

#[command]
pub async fn word(ctx: &Context, msg: &Message) -> CommandResult {
    let url = "https://www.dictionary.com/e/word-of-the-day/";
    let dt = Utc::today().format("%B %d, %Y");
    let colors: Vec<i32> = vec![
        0x00ffff, 0x9fe2bf, 0xccccff, 0xdfff00, 0xf08080, 0xeb984e, 0xff8b3d, 0xffaf7a, 0xf8b195,
        0xf67280, 0xcd6c84, 0x6c587b, 0x355c7d, 0xa8e6ce, 0xff8c94,
    ];
    let color = colors.choose(&mut rand::thread_rng()).unwrap();

    let res = reqwest::get(url).await?.text().await?;

    let test: Word = scrape(&res);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} | {}", test.word, dt))
                    .colour(*color)
                    .field("Pronounciation", test.pronounciation, false)
                    .field("Word type", format!("*{}*", test.word_type), false)
                    .field("Definition", test.definition, false)
                    .field("Example", test.example, false)
                    .footer(|f| {
                        f.text("Word of the Day");
                        f
                    })
            })
        })
        .await?;

    Ok(())
}

pub fn scrape(res: &str) -> Word {
    let body = Html::parse_document(&res);

    // word scraping
    let word_selector = Selector::parse(r#"div[class="otd-item-headword__word"]"#).unwrap();
    let _word = body
        .select(&word_selector)
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // pronounciation scraping
    let pronounciation_selector =
        Selector::parse(r#"div[class="otd-item-headword__pronunciation"]"#).unwrap();
    let bold_word = Selector::parse(r#"span[class="bold"]"#).unwrap();
    let italic_word = Selector::parse(r#"span[class="italic"]"#).unwrap();
    let pronounciation_div = body.select(&pronounciation_selector).next().unwrap();
    let mut pronounciation = pronounciation_div
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    // checks if the word is bolded or italicized and sets correct markdown styles
    for bw in pronounciation_div.select(&bold_word) {
        let bolded = bw.text().collect::<String>().trim().to_string();
        pronounciation = pronounciation.replace(&bolded, &format!("**{}**", bolded));
    }
    for iw in pronounciation_div.select(&italic_word) {
        let italicized = iw.text().collect::<String>().trim().to_string();
        pronounciation = pronounciation.replace(&italicized, &format!("*{}*", italicized));
    }

    // definition scraping
    let def_selector = Selector::parse(r#"div[class="otd-item-headword__pos"]"#).unwrap();
    let def_div_selector = Selector::parse("p").unwrap();
    let definition = body
        .select(&def_selector)
        .next()
        .unwrap()
        .select(&def_div_selector)
        .skip(1)
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // word type scraping
    let word_type_selector = Selector::parse(r#"span[class="luna-pos"]"#).unwrap();
    let word_type = body
        .select(&word_type_selector)
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // example scraping
    let example_selector = Selector::parse(r#"div[class="wotd-item-example__content"]"#).unwrap();
    let example_base: String = body
        .select(&example_selector)
        .next()
        .unwrap()
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    let example = example_base.replace(&_word, &format!("**{}**", _word));

    Word {
        word: _word,
        pronounciation: pronounciation,
        word_type: word_type,
        definition: definition,
        example: example,
    }
}
